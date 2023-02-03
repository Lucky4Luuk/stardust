#[macro_use] extern crate log;

use std::time::Instant;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;

use rayon::prelude::*;
use foxtail::prelude::*;

use stardust_common::camera::Camera;
use stardust_common::math::*;
use stardust_ecs::prelude::*;
use stardust_world::GpuModel;

pub mod renderer;
pub mod widgets;
pub mod resource_manager;

use widgets::*;
use resource_manager::*;

pub struct EngineInternals {
    world: stardust_world::World,
    renderer: renderer::Renderer,
    framebuffer: Framebuffer,
    render_size: (u32, u32),
    render_offset: (u32, u32),

    camera: Camera,
    delta_s: f32,
    frame_counter: usize,
    cam_rot_y: f32,
    last_frame: Instant,

    pub resources: ResourceManager,
    pub current_scene: Scene,
    pub current_scene_path: Option<PathBuf>,

    pub console_pending_writes: VecDeque<String>,
    pub selected_entity: Option<Entity>,
}

pub struct Engine {
    widgets: WidgetManager,
    internals: EngineInternals,
}

impl Engine {
    fn new(ctx: &mut Context) -> Self {
        ctx.set_maximized(true);

        let widgets = WidgetManager::new();

        ctx.set_window_title("Stardust engine");
        trace!("Demo created!");
        let world = stardust_world::World::new(ctx);
        let renderer = renderer::Renderer::new(ctx);
        let mut camera = Camera::default();
        camera.pos = vec3(1024.0, 1024.0, 1624.0);
        camera.rotation = Quat::from_rotation_y(0.0);

        let render_size = ctx.size();

        let mut obj = Self {
            widgets,
            internals: EngineInternals {
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

                resources: ResourceManager::new(),
                current_scene: Scene::new(),
                current_scene_path: None,

                console_pending_writes: VecDeque::new(),
                selected_entity: None,
            },
        };

        obj.widgets.add_widget(Box::new(FsBrowser::new()), DockLoc::Bottom);
        obj.widgets.add_widget(Box::new(Console::new()), DockLoc::Bottom);

        obj.widgets.add_widget(Box::new(SceneHierachy::new()), DockLoc::Left);
        obj.widgets.add_widget(Box::new(Inspector::new()), DockLoc::Right);

        obj.widgets.add_widget(Box::new(PerfDebug), DockLoc::Floating);

        let voxels: Vec<(stardust_common::voxel::Voxel, UVec3)> = (0..=15).into_par_iter().map(|x| {
            let mut voxels = Vec::new();
            for y in 0..=15 {
                for z in 0..=15 {
                    let v = stardust_common::voxel::Voxel::new([x,y,z], 255, 0, false, 255);
                    let p = uvec3(x as u32 + 1024 + 256,y as u32 + 1024,z as u32 + 1024);
                    voxels.push((v, p));
                }
            }
            voxels
        }).flatten().collect();
        voxels.into_iter().for_each(|(v, p)| {
            obj.internals.world.set_voxel(v, p);
        });
        obj.internals.world.process(ctx);

        obj
    }
}

impl Deref for Engine {
    type Target = EngineInternals;
    fn deref(&self) -> &Self::Target {
        &self.internals
    }
}

impl DerefMut for Engine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.internals
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

    fn update(&mut self, ctx: &mut Context) {
        let now = Instant::now();
        let elapsed = now - self.last_frame;
        self.delta_s = elapsed.as_secs_f32();
        self.last_frame = now;

        // Refresh ResourceManager if needed
        if self.internals.resources.request_refresh {
            self.widgets.add_widget(Box::new(ResourceLoader::new(&mut self.internals, false)), DockLoc::Floating);
            self.internals.resources.request_refresh = false;
        }

        self.camera.rotation = Quat::from_rotation_y(self.cam_rot_y);

        self.internals.current_scene.update(self.internals.delta_s);

        self.internals.current_scene.update_dirty_models(&self.internals.world);
    }

    fn render(&mut self, ctx: &mut Context) {
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
            // Draw docked widgets
            self.widgets.draw_docked(ctx, egui_ctx, &mut self.internals);

            // Draw floating windows
            self.widgets.draw_floating(ctx, egui_ctx, &mut self.internals);

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

impl Engine {
    fn console_write<S: Into<String>>(&mut self, s: S) {
        self.console_pending_writes.push_back(s.into());
    }
}

fn main() {
    foxtail::run(|ctx| Engine::new(ctx))
}
