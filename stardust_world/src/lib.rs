#[macro_use]
extern crate log;

use std::sync::{Arc, Mutex};
use foxtail::prelude::*;

use stardust_common::math::*;
use stardust_common::voxel::Voxel;

pub mod layer0;
pub mod brick;

use layer0::*;
use brick::*;

pub const BRICK_POOL_SIZE: usize = 32768;
pub const LAYER0_POOL_SIZE: usize = 8192;
const BRICK_MAP_SIZE: usize = 64;
const VOXEL_QUEUE_SIZE: usize = 4096;

pub struct World {
    brick_pool: FixedSizeBuffer<Brick>,
    layer0_pool: FixedSizeBuffer<Layer0>,
    layer0_map: FixedSizeBuffer<u32>,

    voxel_queue: Arc<Mutex<Vec<(Voxel, UVec3)>>>,
    voxel_queue_gpu: FixedSizeBuffer<[u32; 4]>,

    brick_pool_counter: AtomicCounter,
    layer0_pool_counter: AtomicCounter,

    cs_process_voxels: ComputeShader,
    cs_alloc_layers: ComputeShader,
    cs_alloc_bricks: ComputeShader,
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

        let mut obj = Self {
            brick_pool,
            layer0_pool,
            layer0_map,

            voxel_queue: Arc::new(Mutex::new(Vec::new())),
            voxel_queue_gpu,

            brick_pool_counter,
            layer0_pool_counter,

            cs_process_voxels,
            cs_alloc_layers,
            cs_alloc_bricks,
        };

        // let voxels: Vec<(stardust_common::voxel::Voxel, UVec3)> = (0..=255).into_iter().map(|x| {
        //     let mut voxels = Vec::new();
        //     for y in 0..=255 {
        //         for z in 0..=255 {
        //             let v = stardust_common::voxel::Voxel::new([x,y,z], 255, 0, false, 0);
        //             let p = uvec3(x as u32 + 1024,y as u32 + 1024,z as u32 + 1024);
        //             voxels.push((v, p));
        //         }
        //     }
        //     voxels
        // }).flatten().collect();
        // voxels.into_iter().for_each(|(v, p)| {
        //     obj.set_voxel(v, p);
        // });
        // for _ in 0..2 {
        //     for ix in 0..128 {
        //         for iy in 0..128 {
        //             for iz in 0..128 {
        //                 // let cx = (ix % 16) as u8;
        //                 // let cy = (iy % 16) as u8;
        //                 // let c = [cx * 16, cy * 16, 255];
        //                 // let o = 0;
        //                 // let v = Voxel::new(c, 255, 0, false, o);
        //                 let v = Voxel::empty();
        //                 obj.set_voxel(
        //                     v,
        //                     uvec3(
        //                         ix + 1024,
        //                         iy + 1024,
        //                         iz + 1024,
        //                     ),
        //                 );
        //             }
        //         }
        //     }
        //     obj.process(ctx);
        // }
        for ix in 0..128 {
            for iy in 0..128 {
                for iz in 0..128 {
                    let cx = (ix % 16) as u8;
                    let cy = (iy % 16) as u8;
                    let c = [cx * 16, cy * 16, 255];
                    let ox = ix as i16 - 64;
                    let oy = iy as i16 - 64;
                    let oz = iz as i16 - 64;
                    let v = if ox * ox + oy * oy + oz * oz > 57 * 57 {
                        Voxel::empty()
                    } else {
                        Voxel::new(c, 255, 0, false, 255)
                    };
                    obj.set_voxel(
                        v,
                        uvec3(
                            ix + 1024,
                            iy + 1024,
                            iz + 1024,
                        ),
                    );
                }
            }
        }

        obj.process(ctx);
        obj
    }

    pub fn voxels_queued(&self) -> usize {
        self.voxel_queue.lock().unwrap().len()
    }

    pub fn set_voxel(&self, voxel: Voxel, world_pos: UVec3) {
        puffin::profile_function!();
        let mut lock = self.voxel_queue.lock().unwrap();
        lock.push((voxel, world_pos));
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

    pub fn process(&mut self, ctx: &Context) {
        puffin::profile_function!();

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

            lock.clear();
        }

        for slice in write_total {
            let size = slice.len();
            self.write_queue_to_gpu(slice);
            // self.brick_pool_counter.reset(0);
            // self.layer0_pool_counter.reset(0);

            self.bind();
            self.voxel_queue_gpu.bind(3);
            self.brick_pool_counter.bind(4);
            self.layer0_pool_counter.bind(5);

            self.cs_alloc_layers.dispatch([size as u32, 1, 1]);
            ctx.fence();
            self.cs_alloc_bricks.dispatch([size as u32, 1, 1]);
            ctx.fence();
            self.cs_process_voxels.dispatch([size as u32, 1, 1]);

            self.layer0_pool_counter.unbind();
            self.brick_pool_counter.unbind();
            self.voxel_queue_gpu.unbind();
            self.unbind();
        }

        self.voxel_queue_gpu.clear();
    }
}
