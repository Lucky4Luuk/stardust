#[macro_use] extern crate log;
use foxtail::prelude::*;

pub mod renderer;

pub struct Engine {
    world: stardust_world::World,
    renderer: renderer::Renderer,
}

impl Engine {
    fn new(ctx: &Context) -> Self {
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
    fn update(&mut self, ctx: &Context) {}
    fn render(&mut self, ctx: &Context) {
        // self.world.process();
        self.renderer.render(ctx, &mut self.world);
    }
}

fn main() {
    foxtail::run(|ctx| Engine::new(ctx))
}
