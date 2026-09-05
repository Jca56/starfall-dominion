use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::AppResult;
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
            label: Some("Viewport size and compositor scale"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Starfield viewport"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: viewport.as_entire_binding(),
            }],
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
        // Encode one WGSL vec4 without an external byte-casting crate or unsafe code.
        let values = [
            self.config.width as f32,
            self.config.height as f32,
            window.scale_factor() as f32,
            0.0,
        ];
        let mut bytes = [0_u8; 16];
        for (destination, value) in bytes.chunks_exact_mut(4).zip(values) {
            destination.copy_from_slice(&value.to_le_bytes());
        }
        self.gpu.queue.write_buffer(&self.viewport, 0, &bytes);
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
