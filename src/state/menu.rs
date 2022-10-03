use std::collections::VecDeque;
use std::f32::consts::PI;
use std::io::Cursor;
use std::time::Duration;

use egui::{Button, Color32, Context, Frame, Image, Pos2, Rect, Slider, Vec2};
use kira::{LoopBehavior, Volume};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::tween::{Easing, Tween};
use rand::{Rng, thread_rng};
use specs::WorldExt;
use winit::event::VirtualKeyCode;

use crate::engine::{GameState, LoopState, StateData, StateEvent, Trans};
use crate::engine::invert_color::InvertColorRenderer;
use crate::engine::point::{PointRenderer, PointVertexData};

#[derive(Default)]
pub struct QuestionSpellCard {
    ps: VecDeque<PointVertexData>,
    delta: VecDeque<(f32, f32)>,
    angle: f32,
    a: f32,
}

impl QuestionSpellCard {
    fn create_bullet(&mut self, angle: f32, center: [f32; 2]) {
        let d = (angle * PI / 180.0).sin_cos();
        self.delta.push_back(d);
        let mut rng = thread_rng();
        let r = rng.gen();
        let g = rng.gen();
        let b = rng.gen();
        self.ps.push_back(PointVertexData {
            color: [r, g, b, 1.0],
            pos: center,
        });
    }
}

pub struct MainMenu {
    win_target: f32,
    bg: Option<egui::TextureHandle>,
    left_color: [f32; 3],
    right_color: [f32; 3],
    vol: f32,
    handle: Option<StaticSoundHandle>,
    sp: QuestionSpellCard,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            win_target: 100.0,
            bg: None,
            left_color: [212.0 / 255.0, 205.0 / 255.0, 241.0 / 255.0],
            right_color: [0.75, 0.0, 0.0],
            vol: 0.5,
            handle: None,
            sp: Default::default(),
        }
    }
}

impl MainMenu {}

impl GameState for MainMenu {
    fn start(&mut self, s: &mut StateData) {
        if let Some(gpu) = &s.window.gpu {
            s.window.world.insert(InvertColorRenderer::new(gpu));
            s.window.world.insert(PointRenderer::new(gpu));
        }
        if let Some(al) = &mut s.window.audio {
            let music_data = include_bytes!("../../sign/th08_18.mp3");
            let mut s = StaticSoundSettings::default();
            s.loop_behavior = Some(LoopBehavior { start_position: 0.0 });
            let handle = al.manager.play(StaticSoundData::from_cursor(Cursor::new(music_data),
                                                                      s).expect("Read sound data failed"))
                .expect("Play bgm failed");
            self.handle = Some(handle);
        }
    }

    fn update(&mut self, s: &mut StateData) -> (Trans, LoopState) {
        if s.window.inputs.is_pressed(&[VirtualKeyCode::S]) {
            s.window.inputs.pressed_any_cur_frame = 0;
            (Trans::Push(Box::new(super::ClickState::default())), LoopState::POLL)
        } else {
            (Trans::None, LoopState::POLL)
        }
    }


    fn render(&mut self, s: &mut StateData, ctx: &Context) -> Trans {
        let mut ret = Trans::None;
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let size = Vec2::new(ui.max_rect().width() / 4.0, ui.max_rect().height() / 4.0);
                ui.allocate_ui_at_rect(Rect {
                    min: Default::default(),
                    max: Pos2::new(ui.max_rect().width(), ui.max_rect().height() - 600.0 * ui.available_height() / 900.0),
                }, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.heading("Win Target:");
                        ui.add(Slider::new(&mut self.win_target, 100.0..=1000.0));
                        let mut started = false;
                        if ui.add_sized(size, Button::new("Start")).clicked() {
                            started = true;
                        }
                        if let Some(h) = &mut self.handle {
                            ui.heading("BGM Vol:");
                            if ui.add(Slider::new(&mut self.vol, 0.0..=1.0)).changed() {
                                println!("Changed");
                                h.set_volume(Volume::Amplitude(self.vol as _), Tween {
                                    start_time: Default::default(),
                                    duration: Duration::from_secs(0),
                                    easing: Easing::Linear,
                                }).unwrap();
                            }
                        }
                        if s.window.inputs.is_pressed(&[VirtualKeyCode::Return]) {
                            started = true;
                        }

                        if started {
                            ret = Trans::Push(Box::new(super::MulClickState::new(self.win_target, ui, self.left_color, self.right_color)))
                        }
                    });
                });
                ui.horizontal_centered(|ui| {
                    ui.heading("Left Color:");
                    ui.color_edit_button_rgb(&mut self.left_color);
                    ui.add_space(size.x);
                    ui.heading("Right Color:");
                    ui.color_edit_button_rgb(&mut self.right_color);
                })
            });
        let (w, h) = {
            let cfg = &s.window.gpu.as_ref().unwrap().surface_cfg;
            (cfg.width as f32, cfg.height as f32)
        };
        // run spellcard
        let sp = &mut self.sp;
        let center = [w / 2.0, h / 2.0];
        for i in 0..3 {
            sp.create_bullet(sp.angle + i as f32 * 120.0, center);
        }
        while let Some(fst) = sp.ps.front() {
            if fst.pos[0] < -100.0 || fst.pos[1] < -100.0 || fst.pos[0] > w + 100.0 || fst.pos[1] > h + 100.0 {
                sp.ps.pop_front();
                sp.delta.pop_front();
            } else {
                break;
            }
        }
        for x in sp.ps.iter_mut().zip(sp.delta.iter()) {
            x.0.pos[0] += x.1.0 * 300.0 * s.dt;
            x.0.pos[1] += x.1.1 * 300.0 * s.dt;
        }
        let pr = s.window.world.read_resource::<PointRenderer>();
        let slice = sp.ps.as_slices();
        pr.render(&s.window, &s.window.render.as_ref().unwrap().views.get_screen().view, slice.0);
        pr.render(&s.window, &s.window.render.as_ref().unwrap().views.get_screen().view, slice.1);
        sp.a += s.dt * 9.0;
        sp.a %= 360.0;
        sp.angle += sp.a;
        sp.angle %= 360.0;
        ret
    }

    fn shadow_render(&mut self, _: &StateData, ctx: &Context) {
        egui::CentralPanel::default()
            .frame(Frame::none()).show(ctx, |ui| {
            let tex = self.bg.get_or_insert_with(|| {
                ui.ctx().load_texture("bg",
                                      crate::engine::assets::load_image_from_memory(include_bytes!("../../sign/bg.png")).unwrap(),
                                      egui::TextureFilter::Linear)
            });
            let a = 32;
            ui.add(Image::new(tex.id(), ui.max_rect().max.to_vec2())
                .tint(Color32::from_rgba_premultiplied(a, a, a, a)));
        });
    }

    fn on_event(&mut self, s: Option<&mut StateData>, e: StateEvent) {
        if matches!(e, StateEvent::FoundGPU) {
            let s = s.unwrap();
            if !s.window.world.has_value::<InvertColorRenderer>() {
                if let Some(gpu) = &s.window.gpu {
                    s.window.world.insert(InvertColorRenderer::new(gpu));
                }
            }
            if !s.window.world.has_value::<PointRenderer>() {
                if let Some(gpu) = &s.window.gpu {
                    s.window.world.insert(PointRenderer::new(gpu));
                }
            }
            if self.bg.is_some() {
                self.bg = None;
                self.bg.get_or_insert_with(|| {
                    s.window.egui_ctx.load_texture("bg",
                                                   crate::engine::assets::load_image_from_memory(include_bytes!("../../sign/bg.png")).unwrap(),
                                                   egui::TextureFilter::Linear)
                });
            }
        }
    }
}
