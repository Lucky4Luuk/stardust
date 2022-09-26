#[macro_use] extern crate log;
#[macro_use] extern crate sensible_env_logger;

use std::ops::Deref;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

pub mod rendering;

pub trait App {
    fn update(&self, ctx: &Context) {}
    fn render(&self, ctx: &Context) {}
}

struct State<A: App> {
    app: A,
    renderer: rendering::Renderer,
}

impl<A: App> State<A> {
    fn new<F: Fn(&Context) -> A>(window: &Window, f: F) -> Self {
        let renderer = rendering::Renderer::new(window);

        let ctx = Context::new(&renderer);
        let app = f(&ctx);
        drop(ctx);

        Self {
            app: app,
            renderer: renderer,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    // TODO: Actually process events here
    fn event(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        if !self.renderer.is_context_current {
            self.renderer.context.make_current();
            self.renderer.is_context_current = true;
        }
        let ctx = Context::new(&self.renderer);
        self.app.update(&ctx);
        drop(ctx);
        if self.renderer.is_context_current {
            self.renderer.context.make_not_current();
            self.renderer.is_context_current = false;
        }
    }

    fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.renderer.size()
    }

    fn render(&mut self) -> Result<(), rendering::RenderError> {
        self.renderer.render()?;
        Ok(())
    }
}

// Contains references to parts of the current state, for use
// in the user facing API
pub struct Context<'c> {
    renderer: &'c rendering::Renderer,
}

impl<'c> Context<'c> {
    fn new(renderer: &'c rendering::Renderer) -> Self {
        Self {
            renderer: renderer,
        }
    }
}

impl<'c> Deref for Context<'c> {
    type Target = rendering::Renderer;
    fn deref(&self) -> &Self::Target {
        &self.renderer
    }
}

pub fn run<A: App + 'static, F: Fn(&Context) -> A>(f: F) {
    init_timed!();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window, f);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, window_id } if window_id == window.id() => if !state.event(event) {
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                },
                _ => {}
            }
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(e) => eprintln!("{:?}", e),
            }
        },
        Event::MainEventsCleared => {
            state.update();
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}
