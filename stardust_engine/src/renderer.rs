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
        let shader = shader::Shader::new(&ctx, (VS, "../shaders/vs.glsl"), (FS, "../shaders/fs.glsl"));
        debug!("Renderer created!");
        Self {
            mesh,
            shader,
        }
    }

    pub fn render(&self, _ctx: &Context, world: &mut World, camera: &Camera, render_size: (u32, u32)) {
        puffin::profile_function!();
        let aspect_ratio = (render_size.0 as f32) / (render_size.1 as f32);
        self.shader.while_bound(|uni| {
            puffin::profile_scope!("raytracing");
            world.bind();
            let m = camera.matrix_invprojview(aspect_ratio).to_cols_array();
            uni.set_mat4("invprojview", m);
            uni.set_vec3("rayPos", camera.pos.into());
            self.mesh.draw()?;
            world.unbind();
            Ok(())
        }).expect("Failed to render!");
    }
}
