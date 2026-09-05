use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

use crate::AppResult;
use crate::gpu::Renderer;
use crate::interface::Interface;

#[derive(Default)]
pub(crate) struct App {
    renderer: Option<Renderer>,
    window: Option<Arc<Window>>,
    error: Option<Box<dyn std::error::Error>>,
    retry_at: Option<Instant>,
    surface_failures: u8,
    presented: bool,
    interface: Interface,
}

impl App {
    pub(crate) fn finish(self) -> AppResult<()> {
        self.error.map_or(Ok(()), Err)
    }

    fn open(&mut self, event_loop: &ActiveEventLoop) -> AppResult<()> {
        let attributes = Window::default_attributes()
            .with_title("Starfall Dominion")
            .with_decorations(false)
            .with_inner_size(LogicalSize::new(1280.0, 800.0))
            .with_min_inner_size(LogicalSize::new(900.0, 640.0));
        let window = Arc::new(event_loop.create_window(attributes)?);
        let renderer = Renderer::new(Arc::clone(&window), &self.interface.text)?;
        let size = window.inner_size();
        eprintln!(
            "Window: {} × {} physical pixels; compositor scale {:.2}",
            size.width,
            size.height,
            window.scale_factor()
        );
        window.request_redraw();
        self.renderer = Some(renderer);
        self.window = Some(window);
        Ok(())
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl Into<Box<dyn std::error::Error>>) {
        self.error = Some(error.into());
        event_loop.exit();
    }

    fn draw(&mut self, event_loop: &ActiveEventLoop) {
        let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) else {
            return;
        };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            self.retry_at = None;
            return;
        }
        renderer.resize(size);
        self.interface.rebuild(size, window.scale_factor());
        window.set_cursor(self.interface.cursor());
        if self.interface.needs_rebuild() {
            window.request_redraw();
        }
        match renderer.draw(window, &mut self.interface) {
            Ok(()) => {
                self.retry_at = None;
                self.surface_failures = 0;
                if !self.presented {
                    eprintln!("Main menu presented with Lantern UI 2. F11: fullscreen · Esc: back");
                    self.presented = true;
                }
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                self.fail(
                    event_loop,
                    "GPU ran out of memory while drawing the starfield",
                );
            }
            Err(error) => {
                self.surface_failures += 1;
                if self.surface_failures >= 5 {
                    self.fail(event_loop, error);
                    return;
                }
                eprintln!("Surface: {error}; retrying shortly");
                if matches!(
                    error,
                    wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated
                ) {
                    renderer.reconfigure();
                }
                self.retry_at = Some(Instant::now() + Duration::from_millis(100));
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none()
            && let Err(error) = self.open(event_loop)
        {
            self.fail(event_loop, error);
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.renderer = None;
        self.window = None;
        self.retry_at = None;
        self.surface_failures = 0;
        self.presented = false;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref().filter(|window| window.id() == id) else {
            return;
        };
        if self.interface.window_event(&event) {
            window.request_redraw();
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => window.request_redraw(),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                eprintln!("Compositor scale changed to {scale_factor:.2}");
                window.request_redraw();
            }
            WindowEvent::Occluded(false) => window.request_redraw(),
            WindowEvent::RedrawRequested => self.draw(event_loop),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                if let PhysicalKey::Code(KeyCode::F11) = event.physical_key {
                    let fullscreen = window
                        .fullscreen()
                        .is_none()
                        .then_some(Fullscreen::Borderless(None));
                    window.set_fullscreen(fullscreen);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(deadline) = self.retry_at {
            if Instant::now() < deadline {
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
                return;
            }
            self.retry_at = None;
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        // A still backdrop only needs another frame after a window event.
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}
