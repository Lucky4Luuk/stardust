#[macro_use] extern crate log;
use std::time::Instant;
use foxtail::prelude::*;

pub mod renderer;

pub struct Engine {
    world: stardust_world::World,
    renderer: renderer::Renderer,
    delta_s: f32,
    last_frame: Instant,
}

impl Engine {
    fn new(ctx: &mut Context) -> Self {
        ctx.set_window_title("Stardust engine");
        trace!("Demo created!");
        let world = stardust_world::World::new(ctx);
        let renderer = renderer::Renderer::new(ctx);
        Self {
            world: world,
            renderer: renderer,
            delta_s: 0.0,
            last_frame: Instant::now(),
        }
    }
}

impl App for Engine {
    fn update(&mut self, ctx: &mut Context) {
        let now = Instant::now();
        let elapsed = now - self.last_frame;
        self.delta_s = elapsed.as_secs_f32();
        self.last_frame = now;
    }

    fn render(&mut self, ctx: &mut Context) {
        // self.world.process();
        self.renderer.render(ctx, &mut self.world);
        let size = ctx.size();
        ctx.draw_ui(|egui_ctx| {
            egui::Window::new("debug window").show(egui_ctx, |ui| {
                ui.heading("Debug");
                ui.label(&format!("FPS: {}", 1.0 / self.delta_s));
                ui.label(&format!("ms: {}", self.delta_s * 1000.0));
                ui.label(&format!("render resolution: {:?}", size));
            });
        });
    }
}

fn main() {
    foxtail::run(|ctx| Engine::new(ctx))
}
