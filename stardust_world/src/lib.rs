// Concept behind voxel management in stardust:
// 1. Stream voxels from the CPU to a fixed length GPU buffer
//    This avoids constant allocation, but has max streaming rate per frame
// 2. From GPU input buffer, put voxels in world buffer using compute shader (world buffer permanently allocated)
//    The world buffer is a brickmap, and uses pooling to avoid allocations
// 3. While rendering a frame, every untouched brick gets freed up and put back into the pool of free bricks
//    to allow new voxels to be allocated again? Also needs to be combined with distance-based unloading

#[macro_use] extern crate log;

use std::sync::Arc;
use foxtail::prelude::*;

use stardust_common::math::*;

pub mod voxel;
pub mod brick;

const CS_SRC: &'static str = include_str!("../shaders/cs_stream_shader.glsl");

pub struct World {
    pub stream_buffer: FixedSizeBuffer<voxel::VoxelWithPos>,
    pub brick_pool: FixedSizeBuffer<brick::Brick>,
    pub brick_map: FixedSizeBuffer<u32>,

    pub stream_shader: ComputeShader,
}

impl World {
    pub fn new(ctx: &Context) -> Self {
        debug!("Creating new world...");
        let stream_buffer = FixedSizeBuffer::new(ctx, 256);

        let mut voxels = Vec::new();
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    let p = uvec3((x * 4) as u32, (y * 4) as u32, (z * 4) as u32);
                    let v = voxel::Voxel::new([32+x*4,32+y*4,32+z*4], 255, false, 255);
                    voxels.push(voxel::VoxelWithPos::from_voxel(v, p));
                }
            }
        }
        stream_buffer.write(0, &voxels);

        debug!("Voxel streaming buffer created!");
        let brick_pool = FixedSizeBuffer::new(ctx, 16384);
        debug!("Brick pool created!");
        let brick_map = FixedSizeBuffer::new(ctx, 64*64*64);
        debug!("Brick map created!");

        let brick = brick::Brick::func(|x,y,z| {
            let x = x as u8;
            let y = y as u8;
            let z = z as u8;
            voxel::Voxel::new([32+x*4,32+y*4,32+z*4], 255, false, 255)
        });
        brick_pool.write(0, &[brick]);
        let mut tmp = Box::new([0u32; 64*64*64]);
        for x in 30..34 {
            for y in 30..34 {
                for z in 30..34 {
                    tmp[x+y*64+z*64*64] = 1;
                }
            }
        }
        brick_map.write(0, &tmp[..]);

        let stream_shader = ComputeShader::new(ctx, CS_SRC);
        debug!("Streaming shader compiled!");

        let mut obj = Self {
            stream_buffer: stream_buffer,
            brick_pool: brick_pool,
            brick_map: brick_map,
            stream_shader: stream_shader,
        };
        obj.process();
        obj
    }

    pub fn bind(&mut self) {
        self.brick_pool.bind(0);
        self.brick_map.bind(1);
    }

    pub fn unbind(&mut self) {
        self.brick_pool.unbind();
        self.brick_map.unbind();
    }

    pub fn process(&mut self) {
        self.stream_buffer.bind(0);
        self.brick_pool.bind(1);
        self.brick_map.bind(2);

        self.stream_shader.dispatch([256, 1, 1]);

        self.stream_buffer.unbind();
        self.brick_pool.unbind();
        self.brick_map.unbind();
    }
}
