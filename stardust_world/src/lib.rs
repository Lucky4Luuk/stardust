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

use voxel::*;
use brick::*;

const BRICK_POOL_SIZE: usize = 32768;
const BRICK_MAP_SIZE: usize = 128;

pub struct World {
    brick_pool: FixedSizeBuffer<Brick>,
    brick_map: FixedSizeBuffer<u32>,

    brick_pool_cpu: Box<[Brick]>,
    brick_pool_flag_map: Box<[BrickFlags]>,
    brick_map_cpu: Box<[u32]>,
}

impl World {
    pub fn new(ctx: &Context) -> Self {
        debug!("Creating new world...");
        let brick_pool = FixedSizeBuffer::new(ctx, BRICK_POOL_SIZE);
        debug!("Brick pool created!");
        let brick_map = FixedSizeBuffer::new(ctx, BRICK_MAP_SIZE*BRICK_MAP_SIZE*BRICK_MAP_SIZE);
        debug!("Brick map created!");

        let mut brick_pool_cpu: Box<[Brick]> = vec![Brick::empty(); BRICK_POOL_SIZE].into_boxed_slice();
        let brick = Brick::func(|x,y,z| {
            let x = x as u8;
            let y = y as u8;
            let z = z as u8;
            let c = [x*16,y*16,255];
            let ox = x as i16 - 8;
            let oy = y as i16 - 8;
            let oz = z as i16 - 8;
            let o = if ox*ox+oy*oy+oz*oz > 23 { 0 } else { 255 };
            voxel::Voxel::new(c, 255, false, o)
        });
        brick_pool_cpu[0] = brick;
        let mut brick_map_cpu: Box<[u32]> = vec![0u32; BRICK_MAP_SIZE*BRICK_MAP_SIZE*BRICK_MAP_SIZE].into_boxed_slice();
        for x in 0..BRICK_MAP_SIZE {
            for y in 0..BRICK_MAP_SIZE {
                for z in 0..BRICK_MAP_SIZE {
                    brick_map_cpu[x+y*BRICK_MAP_SIZE+z*BRICK_MAP_SIZE*BRICK_MAP_SIZE] = 0;
                }
            }
        }

        let mut flag = BrickFlags::empty();
        flag.set_dirty(true);
        let mut brick_pool_flag_map = vec![flag; BRICK_POOL_SIZE].into_boxed_slice();
        brick_pool_flag_map[0].set_dirty(true);
        brick_pool_flag_map[0].set_in_use(true);

        let mut obj = Self {
            brick_pool: brick_pool,
            brick_map: brick_map,

            brick_pool_cpu: brick_pool_cpu,
            brick_pool_flag_map: brick_pool_flag_map,
            brick_map_cpu: brick_map_cpu,
        };
        obj.process();
        obj
    }

    pub fn set_voxel(&mut self, voxel: Voxel, world_pos: UVec3) {
        let brick_pos = world_pos / 16;
        let local_pos = world_pos % 16;
        let brick_pos_1d = brick_pos.x as usize + brick_pos.y as usize * BRICK_MAP_SIZE + brick_pos.z as usize * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
        let brick_pool_idx = self.brick_map_cpu[brick_pos_1d] as usize;
        if brick_pool_idx == 0 {
            // Brick not yet allocated

            // Step 1: Find free brick
            let mut free_brick_idx = 0;
            self.brick_pool_flag_map.iter().enumerate().for_each(|(i, flag)| {
                if !flag.in_use() {
                    free_brick_idx = i;
                }
            });
            if free_brick_idx == 0 {
                error!("Failed to place voxel in world! No free bricks left :(");
                todo!("Resize brick buffer?");
            }

            // Step 2: Allocate brick
        } else {
            // Brick already allocated
            let brick = &mut self.brick_pool_cpu[brick_pool_idx - 1];
            brick.set_voxel(voxel, local_pos);
        }
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
        // self.brick_pool.write(0, &self.brick_pool_cpu[..1]);
        self.brick_pool_flag_map.iter_mut().enumerate().for_each(|(i, flag)| {
            if flag.dirty() {
                self.brick_pool.write(i, &[self.brick_pool_cpu[i]]);
                flag.set_dirty(false);
            }
        });

        // TODO: This is always uploaded, but that's very much overkill and bad for performance lol
        self.brick_map.write(0, &self.brick_map_cpu[..]);
    }
}
