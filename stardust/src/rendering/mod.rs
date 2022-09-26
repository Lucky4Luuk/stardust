use winit::window::Window;
use raw_gl_context::{GlConfig, GlContext};
use glow::*;

mod render_pass;

#[derive(Debug)]
pub enum RenderError {
    Generic,
}

pub struct Renderer {
    size: winit::dpi::PhysicalSize<u32>,
    context: GlContext,
    gl: Context,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let context = unsafe { GlContext::create(window, GlConfig::default()).expect("Failed to create OpenGL context!") };
        let gl = unsafe {
            context.make_current();
            let gl = Context::from_loader_function(|symbol| context.get_proc_address(symbol) as *const _);
            context.make_not_current();
            gl
        };

        Self {
            size: size,
            context: context,
            gl: gl,
        }
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
        }
    }

    pub fn render(&mut self) -> Result<(), RenderError> {
        unsafe {
            self.context.make_current();

            self.gl.clear_color(0.2,0.2,0.2,1.0);
            self.gl.clear(COLOR_BUFFER_BIT);
        }

        self.context.swap_buffers();
        self.context.make_not_current();
        Ok(())
    }
}
