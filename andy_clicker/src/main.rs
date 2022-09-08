use log::LevelFilter;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .parse_default_env()
        .init();
    andy_clicker_core::real_main();
}
