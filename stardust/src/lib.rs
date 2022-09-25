#[macro_use] extern crate log;
#[macro_use] extern crate sensible_env_logger;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

pub mod rendering;

pub trait App {
    fn update(&self) {}
    fn render(&self) {}
}

struct State<A: App> {
    app: A,
    renderer: rendering::Renderer,
}

impl<A: App> State<A> {
    async fn new(window: &Window, app: A) -> Self {
        let renderer = rendering::Renderer::new(window).await;

        Self {
            app: app,
            renderer: renderer,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    fn event(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        self.app.update();
    }

    fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.renderer.size()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render()?;
        Ok(())
    }
}

pub fn run(app: impl App + 'static) {
    init_timed!();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = pollster::block_on(State::new(&window, app));

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
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size()),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
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
