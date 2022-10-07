use foxtail::prelude::*;

use stardust_common::camera::Camera;
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
        debug!("Renderer created!");
        Self {
            mesh: mesh,
            shader: shader,
        }
    }

    pub fn render(&self, ctx: &Context, world: &mut World, camera: &Camera) {
        let size = ctx.size();
        let aspect_ratio = size.width as f32 / size.height as f32;
        ctx.fence();
        let _ = self.shader.while_bound(|uni| {
            world.bind();
            let m = camera.matrix_invprojview(aspect_ratio).to_cols_array();
            uni.set_mat4("invprojview", m);
            uni.set_vec3("rayPos", camera.pos.into());
            self.mesh.draw()?;
            world.unbind();
            Ok(())
        });
    }
}
