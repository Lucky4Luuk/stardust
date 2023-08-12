#[macro_use] extern crate log;

use std::time::Instant;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;

pub use foxtail::prelude::*;

pub use stardust_common::camera::Camera;
pub use stardust_common::math::*;
pub use stardust_ecs::prelude::*;
pub use stardust_world::GpuModel;

pub mod renderer;

pub fn run_app<A: VoxelApp + 'static>() {
    foxtail::run(|ctx| Engine::<A>::new(ctx))
}

pub trait VoxelApp {
    fn new(ctx: &Context, engine: &mut EngineInternals) -> Self;
    fn update(&mut self, input: &Input, engine: &mut EngineInternals) {}
}

pub struct EngineInternals {
    pub world: stardust_world::World,
    pub renderer: renderer::Renderer,
    framebuffer: Framebuffer,
    render_size: (u32, u32),
    render_offset: (u32, u32),

    pub camera: Camera,
    pub delta_s: f32,
    frame_counter: usize,
    cam_rot_y: f32,
    last_frame: Instant,

    pub current_scene: Scene,
}

pub struct Engine<A: VoxelApp> {
    app: A,
    internals: EngineInternals,
}

impl<A: VoxelApp> Engine<A> {
    fn new(ctx: &Context) -> Self {
        ctx.set_maximized(true);

        ctx.set_window_title("Stardust engine");
        trace!("Demo created!");

        let world = stardust_world::World::new(ctx);
        let renderer = renderer::Renderer::new(ctx);
        let mut camera = Camera::default();
        camera.pos = vec3(1024.0, 1024.0, 1624.0);
        camera.rotation = Quat::from_rotation_y(0.0);

        let render_size = ctx.size();

        let mut internals = EngineInternals {
            world,
            renderer,
            framebuffer: Framebuffer::new(ctx),
            render_size: (render_size.width, render_size.height),
            render_offset: (0, 0),

            camera,
            delta_s: 0.0,
            frame_counter: 0,
            cam_rot_y: 0.0,
            last_frame: Instant::now(),

            current_scene: Scene::new(),
        };

        let app = A::new(ctx, &mut internals);

        Self {
            app,
            internals,
        }
    }
}

impl<A: VoxelApp> Deref for Engine<A> {
    type Target = EngineInternals;
    fn deref(&self) -> &Self::Target {
        &self.internals
    }
}

impl<A: VoxelApp> DerefMut for Engine<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.internals
    }
}

impl<A: VoxelApp> App for Engine<A> {
    fn event(&mut self, input: &Input) {
        let movespeed = if input.held_shift() { 50.0 } else { 25.0 };
        let rotspeed = 3.0;

        if input.key_held(KeyCode::Right) {
            self.cam_rot_y += rotspeed * self.delta_s;
        }
        if input.key_held(KeyCode::Left) {
            self.cam_rot_y -= rotspeed * self.delta_s;
        }

        let q = self.camera.rotation.conjugate();
        if input.key_held(KeyCode::W) {
            self.internals.camera.pos += q * vec3(0.0, 0.0, -movespeed) * self.internals.delta_s;
        }
        if input.key_held(KeyCode::S) {
            self.internals.camera.pos += q * vec3(0.0, 0.0, movespeed) * self.internals.delta_s;
        }
        if input.key_held(KeyCode::A) {
            self.internals.camera.pos += q * vec3(-movespeed, 0.0, 0.0) * self.internals.delta_s;
        }
        if input.key_held(KeyCode::D) {
            self.internals.camera.pos += q * vec3(movespeed, 0.0, 0.0) * self.internals.delta_s;
        }
        if input.key_held(KeyCode::Space) {
            self.camera.pos.y += movespeed * self.delta_s;
        }
        if input.key_held(KeyCode::C) {
            self.camera.pos.y -= movespeed * self.delta_s;
        }
    }

    fn update(&mut self, ctx: &Context) {
        let now = Instant::now();
        let elapsed = now - self.last_frame;
        self.delta_s = elapsed.as_secs_f32();
        self.last_frame = now;

        self.camera.rotation = Quat::from_rotation_y(self.cam_rot_y);

        self.internals.current_scene.update(self.internals.delta_s);
        self.internals.current_scene.update_dirty_models(&self.internals.world);
    }

    fn render(&mut self, ctx: &Context) {
        puffin::profile_function!();
        self.frame_counter += 1;

        self.world.process(ctx);

        let size = self.render_size;
        let wsize = ctx.size();

        // Use glViewport to scale the framebuffer output correctly
        // TODO: Implement nice feature for this in foxtail
        unsafe { ctx.gl.viewport(0, 0, self.render_size.0 as i32, self.render_size.1 as i32); }
        // TODO: Render function should instead take a framebuffer to render to
        //       Right now, the render function cannot use framebuffers itself, as
        //       it will lose binding for the original framebuffer!
        //       A nicer solution could also be to keep track of the bound framebuffer
        //       in foxtail, and simply bind it only when calling the draw function on
        //       a drawable. Definitely worth looking into
        self.internals.framebuffer.while_bound(|| {
            self.internals.renderer.render(ctx, &mut self.internals.world, &self.internals.camera, size);
            Ok(())
        }).expect("Failed to draw to framebuffer!");

        // Use glViewport to offset and scale the framebuffer output
        unsafe { ctx.gl.viewport(self.render_offset.0 as i32, self.render_offset.1 as i32, self.render_size.0 as i32, self.render_size.1 as i32); }

        // Draw the framebuffer
        self.internals.framebuffer.draw().expect("Failed to draw framebuffer!");
        // Undo the effects of glViewport
        unsafe { ctx.gl.viewport(0, 0, wsize.width as i32, wsize.height as i32); }

        ctx.draw_ui(|egui_ctx| {
            let available_rect = egui_ctx.available_rect();
            let available_size = (
                (available_rect.max.x - available_rect.min.x) as u32,
                (available_rect.max.y - available_rect.min.y) as u32,
            );

            if available_size != self.render_size {
                self.render_size = available_size;
                self.framebuffer.resize((available_size.0.max(1) as i32, available_size.1.max(1) as i32));
                // self.framebuffer.resize((wsize.width as i32, wsize.height as i32));
                self.render_offset = (available_rect.min.x as u32, wsize.height - available_rect.max.y as u32);
            }
        });
    }
}
