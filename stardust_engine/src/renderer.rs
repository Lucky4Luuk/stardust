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

    pub fn render(&self, ctx: &Context, world: &mut World) {
        ctx.fence();
        self.shader.while_bound(|| {
            world.bind();
            self.mesh.draw()?;
            world.unbind();
            Ok(())
        });
    }
}
