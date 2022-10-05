// Concept behind voxel management in stardust:
// 1. Stream voxels from the CPU to a fixed length GPU buffer
//    This avoids constant allocation, but has max streaming rate per frame
// 2. From GPU input buffer, put voxels in world buffer using compute shader (world buffer permanently allocated)
//    The world buffer is a brickmap, and uses pooling to avoid allocations
// 3. While rendering a frame, every untouched brick gets freed up and put back into the pool of free bricks
//    to allow new voxels to be allocated again? Also needs to be combined with distance-based unloading

#[macro_use] extern crate log;

use foxtail::prelude::*;

pub mod voxel;
pub mod brick;

pub struct World {
    pub stream_buffer: FixedSizeBuffer<voxel::Voxel>,
    pub brick_pool: FixedSizeBuffer<brick::Brick>,
    pub brick_map: FixedSizeBuffer<u32>,
}

impl World {
    pub fn new(ctx: &Context) -> Self {
        debug!("Creating new world...");
        let stream_buffer = FixedSizeBuffer::new(ctx, 256);
        debug!("Voxel streaming buffer created!");
        let brick_pool = FixedSizeBuffer::new(ctx, 1024);
        debug!("Brick pool created!");
        let brick_map = FixedSizeBuffer::new(ctx, 16384);
        debug!("Brick map created!");
        Self {
            stream_buffer: stream_buffer,
            brick_pool: brick_pool,
            brick_map: brick_map,
        }
    }
}
