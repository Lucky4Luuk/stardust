use foxtail::prelude::*;

use stardust_common::math::*;
use stardust_world::*;

const VS: &'static str = include_str!("../shaders/vs.glsl");
const FS: &'static str = include_str!("../shaders/fs.glsl");

pub struct Renderer {
    mesh: mesh::Mesh,
    shader: shader::Shader,
}

impl Renderer {
    pub fn new(ctx: &Context) -> Self {
        let mesh = mesh::Mesh::quad(&ctx);
        let shader = shader::Shader::new(&ctx, VS, FS);
        Self {
            mesh: mesh,
            shader: shader,
        }
    }

    pub fn render(&self, world: &World) {
        self.shader.while_bound(|| {
            self.mesh.draw()?;
            Ok(())
        });
    }
}
