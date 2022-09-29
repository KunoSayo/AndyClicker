use std::collections::HashSet;
use std::default::Default;

use egui::{Context, FontData};
use egui_wgpu::renderer::ScreenDescriptor;
use egui_winit::State;
use log::warn;
use specs::{World, WorldExt};
use wgpu::{Color, CommandEncoderDescriptor, Extent3d, ImageCopyTexture, LoadOp,
           Operations, Origin3d, RenderPassColorAttachment, RenderPassDescriptor, TextureAspect};
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::engine::{BakedInputs, GameState, LoopState, MainRendererData, MainRenderViews, OpenalData, Pointer, ResourcesHandles, StateEvent, Trans, WgpuData};

pub struct WindowInstance {
    pub window: Window,
    pub gpu: Option<WgpuData>,
    pub render: Option<MainRendererData>,
    pub res: ResourcesHandles,
    pub last_render_time: std::time::Instant,
    pub egui_ctx: Context,
    pub egui_state: State,

    pub inputs: BakedInputs,
    pub lua: mlua::Lua,
    pub world: World,

    pub al: Option<OpenalData>,
}

impl WindowInstance {
    pub fn new(window: Window, event_loop: &EventLoop<()>) -> Self {
        let gpu = WgpuData::new(&window).ok();

        let res = ResourcesHandles::default();
        let render = if let Some(gpu) = &gpu {
            Some(MainRendererData::new(gpu, &res))
        } else {
            None
        };
        let rua = mlua::Lua::new();
        let egui_ctx = Context::default();
        egui_ctx.set_pixels_per_point(window.scale_factor() as f32);
        let al = match OpenalData::new() {
            Ok(al) => Some(al),
            Err(e) => {
                warn!("OpenAL load failed for {:?}", e);
                None
            }
        };
        Self {
            window,
            gpu,
            render,
            res,
            last_render_time: std::time::Instant::now(),
            egui_ctx,
            egui_state: State::new(event_loop),
            inputs: Default::default(),
            lua: rua,
            world: World::new(),
            al,
        }
    }
}


pub struct Application {
    window: WindowInstance,
    states: Vec<Box<dyn GameState>>,
    running: bool,
}

macro_rules! get_state {
        ($this: expr) => {crate::engine::state::StateData {
            window: &mut $this.window,
            dt: 0.0
        }};
    }

impl Application {
    pub fn new(window: Window, event_loop: &EventLoop<()>) -> Self {
        Self { window: WindowInstance::new(window, event_loop), states: vec![], running: true }
    }

    fn loop_once(&mut self) -> LoopState {
        profiling::scope!("Loop logic once");
        let mut loop_result = LoopState::WAIT_ALL;


        self.window.inputs.swap_frame();
        {
            let mut state_data = get_state!(self);

            for x in &mut self.states {
                loop_result |= x.shadow_update();
            }
            if let Some(last) = self.states.last_mut() {
                let (tran, l) = last.update(&mut state_data);
                self.process_tran(tran);
                loop_result |= l;
            }
        }

        loop_result
    }

    fn process_tran(&mut self, tran: Trans) {
        let last = self.states.last_mut().unwrap();
        let mut state_data = get_state!(self);

        match tran {
            Trans::Push(mut x) => {
                x.start(&mut state_data);
                self.states.push(x);
            }
            Trans::Pop => {
                last.stop(&mut state_data);
                self.states.pop().unwrap();
            }
            Trans::Switch(x) => {
                last.stop(&mut state_data);
                *last = x;
            }
            Trans::Exit => {
                while let Some(mut last) = self.states.pop() {
                    last.stop(&mut state_data);
                }
                self.running = false;
            }
            Trans::Vec(ts) => {
                for t in ts {
                    self.process_tran(t);
                }
            }
            Trans::None => {}
        }
    }

    fn render_once(&mut self) {
        if let (Some(gpu), Some(render)) = (&self.window.gpu, &mut self.window.render) {
            profiling::scope!("Render pth once");
            let render_now = std::time::Instant::now();
            let render_dur = render_now.duration_since(self.window.last_render_time);
            let dt = render_dur.as_secs_f32();

            let swap_chain_frame
                = gpu.surface.get_current_texture().expect("Failed to acquire next swap chain texture");
            let surface_output = &swap_chain_frame;
            {
                let mut encoder = gpu.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("Clear Encoder") });
                let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &render.views.get_screen().view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
                gpu.queue.submit(Some(encoder.finish()));
            }

            let egui_ctx = &self.window.egui_ctx.clone();
            let full_output = egui_ctx.run(self.window.egui_state.take_egui_input(&self.window.window), |egui_ctx| {
                {
                    let mut state_data = get_state!(self);
                    state_data.dt = dt;


                    for game_state in &mut self.states {
                        game_state.shadow_render(&state_data, egui_ctx);
                    }
                    if let Some(g) = self.states.last_mut() {
                        let tran = g.render(&mut state_data, egui_ctx);
                        self.process_tran(tran);
                    }
                }
            });
            let gpu = self.window.gpu.as_ref().unwrap();
            let render = self.window.render.as_mut().unwrap();
            // render ui output to main screen
            {
                let device = gpu.device.as_ref();
                let queue = gpu.queue.as_ref();
                let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("encoder for egui"),
                });

                let screen_descriptor = ScreenDescriptor {
                    size_in_pixels: [gpu.surface_cfg.width, gpu.surface_cfg.height],
                    pixels_per_point: self.window.window.scale_factor() as f32,
                };
                // Upload all resources for the GPU.

                let egui_rpass = &mut render.egui_rpass;
                let paint_jobs = self.window.egui_ctx.tessellate(full_output.shapes);
                for (id, delta) in &full_output.textures_delta.set {
                    egui_rpass.update_texture(device, queue, *id, &delta);
                }
                egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

                // Record all render passes.
                egui_rpass.execute(
                    &mut encoder,
                    &render.views.get_screen().view,
                    &paint_jobs,
                    &screen_descriptor,
                    None,
                );
                // Submit the commands.
                queue.submit(std::iter::once(encoder.finish()));
                full_output.textures_delta.free.iter().for_each(|id| egui_rpass.free_texture(id));
            }
            {
                let mut sd = get_state!(self);
                sd.dt = dt;
                self.states.iter_mut().for_each(|s| s.on_event(Some(&mut sd), StateEvent::PostUiRender));
            }
            let gpu = self.window.gpu.as_ref().unwrap();
            let render = self.window.render.as_mut().unwrap();

            {
                let mut encoder = gpu.device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Copy buffer to screen commands")
                });
                let size = gpu.get_screen_size();
                encoder.copy_texture_to_texture(ImageCopyTexture {
                    texture: &render.views.get_screen().texture,
                    mip_level: 0,
                    origin: Origin3d::default(),
                    aspect: TextureAspect::All,
                }, ImageCopyTexture {
                    texture: &surface_output.texture,
                    mip_level: 0,
                    origin: Default::default(),
                    aspect: TextureAspect::All,
                }, Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                });
                gpu.queue.submit(Some(encoder.finish()));
            }
            //
            // if self.window.inputs.is_pressed(&[VirtualKeyCode::F11]) {
            //     self.window.save_screen_shots();
            // }
            //
            // self.window.pools.render_pool.try_run_one();
            self.window.last_render_time = render_now;
            swap_chain_frame.present();
            self.window.egui_state.handle_platform_output(&self.window.window, &self.window.egui_ctx, full_output.platform_output);
        }
    }

    pub fn run_loop(mut self, event_loop: EventLoop<()>, mut start: impl GameState) {
        start.start(&mut get_state!(&mut self));
        self.states.push(Box::new(start));
        let mut game_draw_requested = false;
        let mut pressed_keys = HashSet::new();
        let mut released_keys = HashSet::new();

        event_loop.run(move |event, _, control_flow| {
            if let Event::WindowEvent { event, .. } = &event {
                self.window.egui_state.on_event(&self.window.egui_ctx, event);
                for x in &mut self.states {
                    x.on_event(None, StateEvent::Window(event));
                }
            }
            match event {
                Event::NewEvents(_) => {
                    profiling::finish_frame!();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit
                }
                Event::WindowEvent {
                    event: WindowEvent::Destroyed,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit
                }
                Event::Resumed => {
                    if self.window.gpu.is_none() {
                        log::info!("gpu not found, try to init");
                        self.window.gpu = WgpuData::new(&self.window.window).ok();
                        if let Some(gpu) = &self.window.gpu {
                            self.window.render = Some(MainRendererData::new(gpu, &self.window.res));
                        }
                        self.window.egui_ctx = Context::default();
                        let mut size = self.window.window.inner_size();
                        self.window.egui_state.on_event(&self.window.egui_ctx, &WindowEvent::Resized(size));
                        self.window.egui_state.on_event(&self.window.egui_ctx, &WindowEvent::ScaleFactorChanged {
                            scale_factor: self.window.window.scale_factor(),
                            new_inner_size: &mut size,
                        });
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(size), ..
                } => {
                    if size.width > 0 || size.height > 0 {
                        if let Some(gpu) = &mut self.window.gpu {
                            gpu.resize(size.width, size.height);
                            if let Some(render) = &mut self.window.render {
                                render.views = MainRenderViews::new(gpu);
                            } else {
                                self.window.render = Some(MainRendererData::new(gpu, &self.window.res));
                            }
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput {
                        input,
                        is_synthetic,
                        ..
                    }, ..
                } => {
                    if !is_synthetic {
                        if let Some(key) = input.virtual_keycode {
                            match input.state {
                                ElementState::Pressed => {
                                    pressed_keys.insert(key);
                                }
                                ElementState::Released => {
                                    released_keys.insert(key);
                                }
                            }
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::Touch(touch), ..
                } => {
                    self.window.inputs.points.insert(touch.id, Pointer::from(touch));
                }
                Event::RedrawRequested(_) => {
                    if !game_draw_requested {
                        log::trace!("System Redraw Requested");
                    }
                    self.render_once();
                    game_draw_requested = false;
                }
                Event::MainEventsCleared => {
                    if !pressed_keys.is_empty() || !released_keys.is_empty() {
                        log::trace!(target: "InputTrace", "process pressed_key {:?} and released {:?}", pressed_keys, released_keys);
                        self.window.inputs.process(&pressed_keys, &released_keys);
                        pressed_keys.clear();
                        released_keys.clear();
                    }
                    if self.running {
                        let LoopState {
                            control_flow: c_f,
                            render
                        } = self.loop_once();
                        if render {
                            game_draw_requested = true;
                            self.window.window.request_redraw();
                        }
                        *control_flow = c_f;
                    } else {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => {}
            }
        });
    }
}



