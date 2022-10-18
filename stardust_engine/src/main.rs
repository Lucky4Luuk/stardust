#[macro_use]
extern crate log;
use foxtail::prelude::*;
use std::time::Instant;

use stardust_common::camera::Camera;
use stardust_common::math::*;

pub mod renderer;

pub struct Engine {
    world: stardust_world::World,
    renderer: renderer::Renderer,
    camera: Camera,
    delta_s: f32,
    frame_counter: usize,
    cam_rot_y: f32,
    last_frame: Instant,
}

impl Engine {
    fn new(ctx: &mut Context) -> Self {
        ctx.set_window_title("Stardust engine");
        trace!("Demo created!");
        let world = stardust_world::World::new(ctx);
        let renderer = renderer::Renderer::new(ctx);
        let mut camera = Camera::default();
        camera.pos = vec3(1024.0, 1024.0, 1624.0);
        camera.rotation = Quat::from_rotation_y(0.0);

        Self {
            world: world,
            renderer: renderer,
            camera: camera,
            delta_s: 0.0,
            frame_counter: 0,
            cam_rot_y: 0.0,
            last_frame: Instant::now(),
        }
    }
}

impl App for Engine {
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
            self.camera.pos += q * vec3(0.0, 0.0, -movespeed) * self.delta_s;
        }
        if input.key_held(KeyCode::S) {
            self.camera.pos += q * vec3(0.0, 0.0, movespeed) * self.delta_s;
        }
        if input.key_held(KeyCode::A) {
            self.camera.pos += q * vec3(-movespeed, 0.0, 0.0) * self.delta_s;
        }
        if input.key_held(KeyCode::D) {
            self.camera.pos += q * vec3(movespeed, 0.0, 0.0) * self.delta_s;
        }
        if input.key_held(KeyCode::Space) {
            self.camera.pos.y += movespeed * self.delta_s;
        }
        if input.key_held(KeyCode::C) {
            self.camera.pos.y -= movespeed * self.delta_s;
        }
    }

    fn update(&mut self, _ctx: &mut Context) {
        // use rand::RngCore;

        let now = Instant::now();
        let elapsed = now - self.last_frame;
        self.delta_s = elapsed.as_secs_f32();
        self.last_frame = now;

        self.camera.rotation = Quat::from_rotation_y(self.cam_rot_y);

        // Add a single random voxel to the world
        // let mut rng = rand::thread_rng();
        // let x = (rng.next_u32() % 16384) / 64 + 1024;
        // let y = (rng.next_u32() % 16384) / 64 + 1024;
        // let z = (rng.next_u32() % 16384) / 64 + 1024;
        // self.world.set_voxel(stardust_world::voxel::Voxel::new([255; 3], 255, 0, false, 255), uvec3(x as u32,y as u32,z as u32));
    }

    fn render(&mut self, ctx: &mut Context) {
        self.frame_counter += 1;

        self.world.process();
        self.renderer.render(ctx, &mut self.world, &self.camera);
        let size = ctx.size();
        ctx.draw_ui(|egui_ctx| {
            egui::Window::new("debug window")
                .resizable(true)
                .show(egui_ctx, |ui| {
                    ui.heading("Debug");
                    ui.label(&format!("fps: {}", 1.0 / self.delta_s));
                    ui.label(&format!("ms: {}", self.delta_s * 1000.0));
                    ui.label(&format!("render resolution: {:?}", size));
                    ui.label(&format!("cam_pos: {:?}", self.camera.pos));
                });
        });
    }
}

fn main() {
    foxtail::run(|ctx| Engine::new(ctx))
}
