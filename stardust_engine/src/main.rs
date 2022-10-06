#[macro_use] extern crate log;
use foxtail::prelude::*;

pub mod renderer;

pub struct Engine {
    world: stardust_world::World,
    renderer: renderer::Renderer,
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
        }
    }
}

impl App for Engine {
    fn update(&mut self, ctx: &mut Context) {}
    fn render(&mut self, ctx: &mut Context) {
        // self.world.process();
        self.renderer.render(ctx, &mut self.world);
        ctx.draw_ui(|egui_ctx| {
            egui::Window::new("side panel").show(egui_ctx, |ui| {
                ui.heading("test");
            });
        });
    }
}

fn main() {
    foxtail::run(|ctx| Engine::new(ctx))
}
