use std::any::type_name;
use std::fmt::{Debug, Formatter};
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use anyhow::anyhow;
use futures::executor::block_on;
use wgpu::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::window::Window;

use crate::engine::{ResourcesHandles, TextureInfo, TextureWrapper};

#[derive(Debug)]
pub struct WgpuData {
    pub surface: Surface,
    pub surface_cfg: SurfaceConfiguration,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub screen_uni_buffer: Buffer,
    pub screen_uni_bind_layout: BindGroupLayout,
    pub screen_uni_bind: BindGroup,

    pub size_scale: [f32; 2],

}

impl WgpuData {
    #[inline]
    pub fn get_screen_size(&self) -> (u32, u32) {
        (self.surface_cfg.width, self.surface_cfg.height)
    }


    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_cfg.width = width;
        self.surface_cfg.height = height;
        self.surface.configure(&self.device, &self.surface_cfg);
        let size = [width as f32, height as f32];
        self.size_scale = [size[0] / 1600.0, size[1] / 900.0];
        self.queue.write_buffer(&self.screen_uni_buffer, 0, bytemuck::cast_slice(&size));
    }

    pub fn new(window: &Window) -> anyhow::Result<Self> {
        let window = AssertUnwindSafe(&window);
        let result = std::panic::catch_unwind(|| {
            log::info!("New graphics state");
            let size = window.inner_size();
            log::info!("Got window inner size {:?}", size);

            let instance = Instance::new(util::backend_bits_from_env().unwrap_or(Backends::PRIMARY));
            log::info!("Got wgpu  instance {:?}", instance);
            log::info!("Window is visible, try surface.");
            let surface = unsafe { instance.create_surface(window.0) };
            log::info!("Created surface {:?}", surface);
            let adapter = block_on(instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: util::power_preference_from_env().unwrap_or(PowerPreference::HighPerformance),
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                })).ok_or(anyhow!("Cannot get adapter"))?;
            log::info!("Got adapter {:?}", adapter);
            let (device, queue) = block_on(adapter
                .request_device(
                    &DeviceDescriptor {
                        label: None,
                        features: Features::empty(),
                        limits: Limits {
                            max_bind_groups: 4,
                            ..Limits::default()
                        },
                    },
                    None,
                ))?;
            let (device, queue) = (Arc::new(device), Arc::new(queue));
            log::info!("Requested device {:?} and queue {:?}", device, queue);

            let formats = surface.get_supported_formats(&adapter);
            log::info!("Adapter chose {:?} for swap chain format", formats);
            let format = if formats.contains(&TextureFormat::Bgra8Unorm) { TextureFormat::Bgra8Unorm } else { formats[0] };
            log::info!("Using {:?} for swap chain format", format);

            let surface_cfg = SurfaceConfiguration {
                usage: TextureUsages::COPY_DST,
                format,
                width: size.width,
                height: size.height,
                present_mode: PresentMode::Fifo,
            };
            surface.configure(&device, &surface_cfg);

            let screen_uni_bind_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
            let size = [size.width as f32, size.height as f32];
            let screen_uni_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&size),
            });
            let screen_uni_bind = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &screen_uni_bind_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &screen_uni_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            });
            let size_scale = [surface_cfg.width as f32 / 1600.0, surface_cfg.height as f32 / 900.0];
            Ok(Self {
                surface,
                surface_cfg,
                device,
                queue,
                screen_uni_buffer,
                screen_uni_bind_layout,
                screen_uni_bind,
                size_scale,
            })
        });
        if let Ok(r) = result {
            return r;
        }
        log::warn!("Failed to get gpu data");
        Err(anyhow!("Get gpu data failed"))
    }
}


#[derive(Debug)]
pub struct MainRenderViews {
    buffers: [TextureWrapper; 2],
    main: usize,
}


pub struct MainRendererData {
    pub staging_belt: util::StagingBelt,
    pub views: MainRenderViews,
    pub egui_rpass: egui_wgpu::renderer::RenderPass,
}

impl Debug for MainRendererData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("views", &self.views)
            .finish()
    }
}

impl MainRendererData {
    pub fn new(gpu: &WgpuData, _handles: &ResourcesHandles) -> Self {
        let staging_belt = util::StagingBelt::new(2048);
        let views = MainRenderViews::new(gpu);
        let egui_rpass = egui_wgpu::renderer::RenderPass::new(&gpu.device, gpu.surface_cfg.format, 1);
        Self {
            staging_belt,
            views,
            egui_rpass,
        }
    }
}


impl MainRenderViews {
    pub fn new(state: &WgpuData) -> Self {
        let size = state.get_screen_size();
        let texture_desc = TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: state.surface_cfg.format,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
        };
        let sampler_desc = SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            ..SamplerDescriptor::default()
        };
        let buffer_a = {
            let texture = state.device.create_texture(&texture_desc);
            let view = texture.create_view(&TextureViewDescriptor::default());

            let sampler = state.device.create_sampler(&sampler_desc);
            TextureWrapper {
                texture,
                view,
                sampler,
                info: TextureInfo::new(size.0, size.1),
            }
        };

        let buffer_b = {
            let texture = state.device.create_texture(&texture_desc);
            let view = texture.create_view(&TextureViewDescriptor::default());

            let sampler = state.device.create_sampler(&sampler_desc);
            TextureWrapper {
                texture,
                view,
                sampler,
                info: TextureInfo::new(size.0, size.1),
            }
        };

        Self {
            buffers: [buffer_a, buffer_b],
            main: 0,
        }
    }

    pub fn get_screen(&self) -> &TextureWrapper {
        &self.buffers[self.main]
    }

    /// Return (src, dst)
    #[allow(unused)]
    pub fn swap_screen(&mut self) -> (&TextureWrapper, &TextureWrapper) {
        let src = self.main;
        self.main = (self.main + 1) & 1;
        let dst = self.main;
        (&self.buffers[src], &self.buffers[dst])
    }
}
