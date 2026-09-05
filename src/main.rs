mod actions;
mod app;
mod camera;
mod gpu;
mod interface;
mod map;
mod sector;
mod world;

use std::process::ExitCode;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

pub(crate) type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Starfall Dominion: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> AppResult<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    app.finish()
}
