#[macro_use] extern crate log;
use stardust::{*, rendering::*};

const VS: &'static str = include_str!("../shaders/vs.glsl");
const FS: &'static str = include_str!("../shaders/fs.glsl");

pub struct Demo {
    mesh: mesh::Mesh,
    shader: shader::Shader,
}

impl Demo {
    fn new(ctx: &Context) -> Self {
        ctx.set_window_title("Stardust demo");
        trace!("Demo created!");
        let mesh = mesh::Mesh::quad(&ctx);
        let shader = shader::Shader::new(&ctx, VS, FS);
        Self {
            mesh: mesh,
            shader: shader,
        }
    }
}

impl App for Demo {
    fn update(&self, ctx: &Context) {}
    fn render(&self, ctx: &Context) {
        self.shader.while_bound(|| {
            self.mesh.draw();
        });
    }
}

fn main() {
    stardust::run(|ctx| Demo::new(ctx))
}
