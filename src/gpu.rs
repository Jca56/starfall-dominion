use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::AppResult;
use crate::fog;
use crate::interface::Interface;
use lntrn_render::{Gpu, Images, Pass2d};
use lntrn_text::TextEngine;

pub(crate) struct Renderer {
    surface: wgpu::Surface<'static>,
    gpu: Gpu,
    ui_pass: Pass2d,
    images: Images,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    viewport: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    fog_texture: wgpu::Texture,
    /// The fog version last uploaded; the interface hands over newer ones.
    fog_version: u64,
}

impl Renderer {
    pub(crate) fn new(window: Arc<Window>, text: &TextEngine) -> AppResult<Self> {
        block_on(Self::initialize(window, text))
    }

    async fn initialize(window: Arc<Window>, text: &TextEngine) -> AppResult<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(Arc::clone(&window))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;
        let info = adapter.get_info();
        eprintln!("GPU: {} ({:?})", info.name, info.backend);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Starfall Dominion device"),
                ..Default::default()
            })
            .await?;

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("GPU cannot present to this window")?;
        if let Some(format) = surface
            .get_capabilities(&adapter)
            .formats
            .into_iter()
            .find(wgpu::TextureFormat::is_srgb)
        {
            config.format = format;
        }
        config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Starfield shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("starfield.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Starfield pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(config.format.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
        let viewport = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Starfield backdrop"),
            size: 48,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let fog_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fog of war"),
            size: fog_extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let fog_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Fog sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Starfield backdrop"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: viewport.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &fog_texture.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&fog_sampler),
                },
            ],
        });

        let gpu = Gpu {
            instance,
            adapter,
            device,
            queue,
        };
        let images = Images::new(&gpu);
        let ui_pass = Pass2d::new(&gpu, config.format, text.atlas(), &images);
        Ok(Self {
            surface,
            gpu,
            ui_pass,
            images,
            config,
            pipeline,
            viewport,
            bind_group,
            fog_texture,
            fog_version: 0,
        })
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        if (self.config.width, self.config.height) != (size.width, size.height) {
            self.config.width = size.width;
            self.config.height = size.height;
            self.reconfigure();
        }
    }

    pub(crate) fn reconfigure(&self) {
        self.surface.configure(&self.gpu.device, &self.config);
    }

    pub(crate) fn draw(
        &mut self,
        window: &Window,
        interface: &mut Interface,
    ) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());
        // Encode three WGSL vec4s without an external byte-casting crate or unsafe code.
        let values = interface.backdrop_uniform(
            PhysicalSize::new(self.config.width, self.config.height),
            window.scale_factor(),
        );
        let mut bytes = [0_u8; 48];
        for (destination, value) in bytes.chunks_exact_mut(4).zip(values) {
            destination.copy_from_slice(&value.to_le_bytes());
        }
        self.gpu.queue.write_buffer(&self.viewport, 0, &bytes);
        if let Some((version, cells)) = interface.fog_texture(self.fog_version) {
            self.gpu.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.fog_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &cells,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(fog::COLUMNS as u32 * 2),
                    rows_per_image: Some(fog::ROWS as u32),
                },
                fog_extent(),
            );
            self.fog_version = version;
        }
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Starfield frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Starfield backdrop"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.ui_pass.draw(
            &self.gpu,
            &mut encoder,
            &view,
            [self.config.width, self.config.height],
            &interface.draw,
            interface.text.atlas_mut(),
            &self.images,
            None,
        );
        self.gpu.queue.submit([encoder.finish()]);
        window.pre_present_notify();
        frame.present();
        Ok(())
    }
}

fn fog_extent() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: fog::COLUMNS as u32,
        height: fog::ROWS as u32,
        depth_or_array_layers: 1,
    }
}

// GPU initialization is our only async work. Park until the future wakes us;
// no async runtime and no busy polling are needed.
fn block_on<T>(future: impl Future<Output = T>) -> T {
    struct ThreadWake(std::thread::Thread);

    impl Wake for ThreadWake {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::park(),
        }
    }
}

#[cfg(test)]
mod tests {
    /// The GPU only checks the shader at launch; catch mistakes headlessly instead.
    #[test]
    fn starfield_shader_parses_and_validates() {
        let source = include_str!("starfield.wgsl");
        let module = wgpu::naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
        wgpu::naga::valid::Validator::new(Default::default(), Default::default())
            .validate(&module)
            .expect("The starfield shader must validate");
    }
}
