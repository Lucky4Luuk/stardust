use std::sync::Arc;
use glow::*;

pub struct Mesh {
    vbo: NativeBuffer,
    vao: NativeVertexArray,
    gl: Arc<Context>,
}

impl super::Drawable for Mesh {
    fn draw(&self) -> Result<(), super::RenderError> {
        unsafe {
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_arrays(TRIANGLES, 0, 3);
            self.gl.bind_vertex_array(None);
        }
        Ok(())
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
        }
    }
}

impl Mesh {
    pub fn quad(renderer: &super::Renderer) -> Self {
        unsafe {
            let quad_vertices = [
                -1.0,-1.0,0.0,
                 1.0,-1.0,0.0,
                 1.0, 1.0,0.0,
                -1.0, 1.0,0.0,
            ];
            let quad_vertices_u8: &[u8] = core::slice::from_raw_parts(
                quad_vertices.as_ptr() as *const u8,
                quad_vertices.len() * core::mem::size_of::<f32>(),
            );

            let gl = renderer.gl.clone();
            trace!("GL cloned!");

            let vbo = gl.create_buffer().expect("Failed to create VBO!");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, quad_vertices_u8, glow::STATIC_DRAW);

            let vao = gl.create_vertex_array().expect("Failed to create VAO!");
            gl.bind_vertex_array(Some(vao));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 8, 0);

            Self {
                vbo: vbo,
                vao: vao,
                gl: gl,
            }
        }
    }
}
