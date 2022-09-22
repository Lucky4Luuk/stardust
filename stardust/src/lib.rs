#[macro_use] extern crate log;
#[macro_use] extern crate sensible_env_logger;

pub fn run(mut app: impl App) {
    init_timed!();
}

pub trait App {
    fn on_update(&self) {}
    fn on_render(&self) {}
}
