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
        let stream_buffer = FixedSizeBuffer::new(ctx, 2048);

        // let mut voxels = Vec::new();
        // for x in 0..64 {
        //     for y in 0..64 {
        //         for z in 0..64 {
        //             let p = uvec3(30 + (x * 4) as u32, 30 + (y * 4) as u32, 30 + (z * 4) as u32);
        //             // let c = [32+x*4,32+y*4,32+z*4];
        //             let c = if p.x > 8 { [0,0,0] } else { [255,255,255] };
        //             let v = voxel::Voxel::new(c, 255, false, 255);
        //             voxels.push(voxel::VoxelWithPos::from_voxel(v, p));
        //         }
        //     }
        // }
        // stream_buffer.write(0, &voxels);

        debug!("Voxel streaming buffer created!");
        let brick_pool = FixedSizeBuffer::new(ctx, 16384);
        debug!("Brick pool created!");
        let brick_map = FixedSizeBuffer::new(ctx, 64*64*64);
        debug!("Brick map created!");

        let brick = brick::Brick::func(|x,y,z| {
            let x = x as u8;
            let y = y as u8;
            let z = z as u8;
            let c = [x*16,y*16,255];
            // let c = [32+x*4,32+y*4,32+z*4];
            // let o = if (x as f32 * 4.0).sin() * 2.0 + 8.0 > (y as f32) { 255 } else { 0 };
            // let o = if y < 8 { 255 } else { 0 };
            // let o = 255;
            let ox = x as i16 - 8;
            let oy = y as i16 - 8;
            let oz = z as i16 - 8;
            let o = if ox*ox+oy*oy+oz*oz > 23 { 0 } else { 255 };
            voxel::Voxel::new(c, 255, false, o)
        });
        brick_pool.write(0, &[brick]);
        let mut tmp = Box::new([0u32; 64*64*64]);
        for x in 0..64 {
            for y in 0..64 {
                for z in 0..64 {
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
