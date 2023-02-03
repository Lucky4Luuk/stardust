#[macro_use]
extern crate log;

use std::sync::{Arc, Mutex};
use foxtail::prelude::*;

use stardust_common::math::*;
use stardust_common::voxel::Voxel;

pub mod layer0;
pub mod brick;
mod data;

use layer0::*;
use brick::*;
pub use data::*;

pub const BRICK_POOL_SIZE: usize = 32768;
pub const LAYER0_POOL_SIZE: usize = 8192;
const BRICK_MAP_SIZE: usize = 64;
const VOXEL_QUEUE_SIZE: usize = 16384;

pub struct World {
    brick_pool: FixedSizeBuffer<Brick>,
    layer0_pool: FixedSizeBuffer<Layer0>,
    layer0_map: FixedSizeBuffer<u32>,

    voxel_queue: Arc<Mutex<Vec<(Voxel, UVec3)>>>,
    voxel_queue_gpu: FixedSizeBuffer<[u32; 4]>,

    model_queue: Arc<Mutex<Vec<(Arc<GpuModel>, UVec3, UVec3)>>>,

    brick_pool_counter: AtomicCounter,
    layer0_pool_counter: AtomicCounter,

    cs_process_voxels: ComputeShader,
    cs_alloc_layers: ComputeShader,
    cs_alloc_bricks: ComputeShader,
    cs_place_model: ComputeShader,

    pub gpu_models: Vec<Arc<GpuModel>>,

    voxels_queued: usize,
    models_queued: usize,
}

impl World {
    pub fn new(ctx: &Context) -> Self {
        debug!("Creating new world...");
        let brick_pool = FixedSizeBuffer::new(ctx, BRICK_POOL_SIZE);
        debug!("GPU Brick pool created!");
        let layer0_pool = FixedSizeBuffer::new(ctx, LAYER0_POOL_SIZE);
        debug!("GPU Layer0 pool created!");
        let layer0_map = FixedSizeBuffer::new(ctx, BRICK_MAP_SIZE * BRICK_MAP_SIZE * BRICK_MAP_SIZE);
        debug!("GPU Brick map created!");
        let voxel_queue_gpu = FixedSizeBuffer::new(ctx, VOXEL_QUEUE_SIZE);
        debug!("GPU Voxel queue created!");
        let brick_pool_counter = AtomicCounter::new(ctx);
        let layer0_pool_counter = AtomicCounter::new(ctx);

        let cs_process_voxels = ComputeShader::new(ctx, include_str!("../shaders/cs_process_voxel_queue.glsl"));
        let cs_alloc_layers = ComputeShader::new(ctx, include_str!("../shaders/cs_alloc_layers.glsl"));
        let cs_alloc_bricks = ComputeShader::new(ctx, include_str!("../shaders/cs_alloc_bricks.glsl"));
        let cs_place_model = ComputeShader::new(ctx, include_str!("../shaders/cs_place_model.glsl"));

        Self {
            brick_pool,
            layer0_pool,
            layer0_map,

            voxel_queue: Arc::new(Mutex::new(Vec::new())),
            voxel_queue_gpu,

            model_queue: Arc::new(Mutex::new(Vec::new())),

            brick_pool_counter,
            layer0_pool_counter,

            cs_process_voxels,
            cs_alloc_layers,
            cs_alloc_bricks,
            cs_place_model,

            gpu_models: Vec::new(),

            voxels_queued: 0,
            models_queued: 0,
        }
    }

    pub fn voxels_queued(&self) -> usize {
        self.voxels_queued
    }

    pub fn models_queued(&self) -> usize {
        self.models_queued
    }

    /// Queues a voxel to be uploaded to the GPU and placed in the world.
    /// Voxel placement order cannot be relied on!
    /// They get uploaded by a compute shader in batches of 4096 voxels, with no ordering within each batch
    /// Batches ARE ordered, however!
    pub fn set_voxel(&self, voxel: Voxel, world_pos: UVec3) {
        puffin::profile_function!();
        let mut lock = self.voxel_queue.lock().unwrap();
        lock.push((voxel, world_pos));
    }

    pub fn update_model(&self, model: Arc<GpuModel>, old_pos: UVec3, new_pos: UVec3) {
        puffin::profile_function!();
        let mut lock = self.model_queue.lock().unwrap();
        lock.push((model, old_pos, new_pos));
    }

    /// Registers a model living in GPU memory. Arc<T> so you can keep a reference to it!
    pub fn register_model(&mut self, model: Arc<GpuModel>) {
        self.gpu_models.push(model);
    }

    pub fn bind(&mut self) {
        self.brick_pool.bind(0);
        self.layer0_pool.bind(1);
        self.layer0_map.bind(2);
    }

    pub fn unbind(&mut self) {
        self.brick_pool.unbind();
        self.layer0_pool.unbind();
        self.layer0_map.unbind();
    }

    fn write_queue_to_gpu(&mut self, write_slice: Vec<[u32; 4]>) {
        puffin::profile_function!();
        self.voxel_queue_gpu.write(0, &write_slice);
    }

    fn process_internal(&mut self, ctx: &Context, size: u32) {
        self.bind();
        self.voxel_queue_gpu.bind(3);
        self.brick_pool_counter.bind(4);
        self.layer0_pool_counter.bind(5);

        self.cs_alloc_layers.dispatch([size, 1, 1]);
        ctx.fence();
        self.cs_alloc_bricks.dispatch([size, 1, 1]);
        ctx.fence();
        self.cs_process_voxels.dispatch([size, 1, 1]);

        self.layer0_pool_counter.unbind();
        self.brick_pool_counter.unbind();
        self.voxel_queue_gpu.unbind();
        self.unbind();

        ctx.fence();
    }

    pub fn process(&mut self, ctx: &Context) {
        puffin::profile_function!();

        self.voxels_queued = self.voxel_queue.lock().unwrap().len();
        self.models_queued = self.model_queue.lock().unwrap().len();

        // Process GPU model changes
        {
            puffin::profile_scope!("process_gpu_models");
            for i in 0..self.models_queued {
                let (model, prev, new) = {
                    let (ref_model, ref_prev, ref_new) = &self.model_queue.lock().unwrap()[i];
                    (Arc::clone(ref_model), *ref_prev, *ref_new)
                };
                for j in 0..2 {
                    let mut offset = 0;
                    let count = model.voxels;
                    'process: loop {
                        let size = (count - offset).min(VOXEL_QUEUE_SIZE);

                        self.voxel_queue_gpu.bind(0);
                        unsafe {
                            ctx.gl.bind_buffer_base(foxtail::glow::SHADER_STORAGE_BUFFER, 1, Some(model.vox_buf.buf()));
                        }

                        self.cs_place_model.set_uniforms(|uni| {
                            uni.set_u32("offset", offset as u32);
                            if j == 0 {
                                uni.set_uvec4("pos", [prev.x, prev.y, prev.z, 0]);
                            } else {
                                uni.set_uvec4("pos", [new.x, new.y, new.z, 1]);
                            }
                        });
                        self.cs_place_model.dispatch([size as u32, 1, 1]);
                        ctx.fence();

                        unsafe {
                            ctx.gl.bind_buffer_base(foxtail::glow::SHADER_STORAGE_BUFFER, 1, None);
                        }
                        self.voxel_queue_gpu.unbind();

                        self.process_internal(ctx, size as u32);

                        offset += VOXEL_QUEUE_SIZE;
                        if offset >= count {
                            break 'process;
                        }
                    }
                }
            }
            self.model_queue.lock().unwrap().clear();
        }

        // Process voxel queue
        {
            puffin::profile_scope!("process_cpu_voxels");
            let mut write_total: Vec<Vec<[u32; 4]>> = Vec::new();
            {
                let mut lock = self.voxel_queue.lock().unwrap();

                for chunk in lock.chunks(VOXEL_QUEUE_SIZE) {
                    let mut write_slice = Vec::new();
                    for (voxel, wpos) in chunk {
                        write_slice.push([wpos.x, wpos.y, wpos.z, voxel.0]);
                    }
                    write_total.push(write_slice);
                }

                // TODO: This doesn't actually free up the allocated memory it seems.
                //       Might be worth looking into freeing up that memory. If users
                //       queue up a lot of voxels, it could permanently take up a lot of memory
                lock.clear();
            }

            for slice in write_total {
                let size = slice.len();
                self.write_queue_to_gpu(slice);
                self.process_internal(ctx, size as u32);
            }
        }

        self.voxel_queue_gpu.clear();
    }
}
