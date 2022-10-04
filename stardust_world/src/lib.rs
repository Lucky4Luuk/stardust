// Concept behind voxel management in stardust:
// 1. Stream voxels from the CPU to a fixed length GPU buffer
//    This avoids constant allocation, but has max streaming rate per frame
// 2. From GPU input buffer, put voxels in world buffer using compute shader (world buffer permanently allocated)
//    The world buffer is mostlikely going to be a brickmap, and uses pooling to avoid allocations
// 3. While rendering a frame, every untouched brick gets freed up and put back into the pool of free bricks
//    to allow new voxels to be allocated again? Also needs to be combined with distance-based unloading

use foxtail::prelude::*;

pub mod voxel;
pub mod brick;

pub struct World {
    stream_buffer: FixedSizeBuffer<voxel::Voxel>,
    brick_map: FixedSizeBuffer<brick::Brick>,
}

impl World {
    pub fn new(ctx: &Context) -> Self {
        let stream_buffer = FixedSizeBuffer::new(ctx, 256);
        let brick_map = FixedSizeBuffer::new(ctx, 1024);
        Self {
            stream_buffer: stream_buffer,
            brick_map: brick_map
        }
    }
}
