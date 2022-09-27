use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::engine::app::Application;

mod engine;
mod state;


pub fn real_main() {
    log::info!("Joined the real main");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Click")
        .with_min_inner_size(PhysicalSize::new(1600, 900))
        .build(&event_loop)
        .unwrap();
    let main = Application::new(window, &event_loop);
    main.run_loop(event_loop, state::MainMenu::default());
}


#[cfg_attr(target_os = "android", ndk_glue::main(logger(level = "info", tag = "andy")))]
pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    real_main();
}