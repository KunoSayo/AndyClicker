use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::engine::app::{Application};

mod engine;
mod state;


#[derive(Default)]
struct ClickData {
    clicks: u128,
}

pub fn real_main() {
    log::info!("Joined the real main");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Andy Clicker")
        .build(&event_loop)
        .unwrap();
    let main = Application::new(window, &event_loop);
    main.run_loop(event_loop, state::MainState::default());
}



#[cfg_attr(target_os = "android", ndk_glue::main(logger(level = "info", tag = "andy")))]
pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    real_main();
}