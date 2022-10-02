#[macro_use] extern crate log;
use foxtail::prelude::*;

const VS: &'static str = include_str!("../shaders/vs.glsl");
const FS: &'static str = include_str!("../shaders/fs.glsl");

pub struct Engine {
    mesh: mesh::Mesh,
    shader: shader::Shader,
}

impl Engine {
    fn new(ctx: &Context) -> Self {
        ctx.set_window_title("Stardust engine");
        trace!("Demo created!");
        let mesh = mesh::Mesh::quad(&ctx);
        let shader = shader::Shader::new(&ctx, VS, FS);
        Self {
            mesh: mesh,
            shader: shader,
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
