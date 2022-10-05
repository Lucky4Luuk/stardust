#[macro_use] extern crate log;
use foxtail::prelude::*;

const VS: &'static str = include_str!("../shaders/vs.glsl");
const FS: &'static str = include_str!("../shaders/fs.glsl");

pub struct Engine {
    mesh: mesh::Mesh,
    shader: shader::Shader,
    world: stardust_world::World,
}

impl Engine {
    fn new(ctx: &Context) -> Self {
        ctx.set_window_title("Stardust engine");
        trace!("Demo created!");
        let mesh = mesh::Mesh::quad(&ctx);
        let shader = shader::Shader::new(&ctx, VS, FS);
        let world = stardust_world::World::new(&ctx);
        Self {
            mesh: mesh,
            shader: shader,
            world: world,
        }
    }
}

impl App for Engine {
    fn update(&mut self, ctx: &Context) {}
    fn render(&mut self, ctx: &Context) {
        self.shader.while_bound(|| {
            self.mesh.draw()?;
            Ok(())
        });
    }
}

fn main() {
    foxtail::run(|ctx| Engine::new(ctx))
}
