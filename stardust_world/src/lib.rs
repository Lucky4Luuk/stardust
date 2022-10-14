#[macro_use]
extern crate log;

use foxtail::prelude::*;

use stardust_common::math::*;

pub mod brick;
pub mod voxel;

use brick::*;
use voxel::*;

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
        let brick_map = FixedSizeBuffer::new(ctx, BRICK_MAP_SIZE * BRICK_MAP_SIZE * BRICK_MAP_SIZE);
        debug!("Brick map created!");

        let brick_pool_cpu: Box<[Brick]> = vec![Brick::empty(); BRICK_POOL_SIZE].into_boxed_slice();
        let brick_map_cpu: Box<[u32]> = vec![0u32; BRICK_MAP_SIZE * BRICK_MAP_SIZE * BRICK_MAP_SIZE].into_boxed_slice();

        let mut flag = BrickFlags::empty();
        flag.set_dirty(true);
        let brick_pool_flag_map = vec![flag; BRICK_POOL_SIZE].into_boxed_slice();

        let mut obj = Self {
            brick_pool: brick_pool,
            brick_map: brick_map,

            brick_pool_cpu: brick_pool_cpu,
            brick_pool_flag_map: brick_pool_flag_map,
            brick_map_cpu: brick_map_cpu,
        };

        for ix in 0..128 {
            for iy in 0..128 {
                for iz in 0..128 {
                    let cx = (ix % 16) as u8;
                    let cy = (iy % 16) as u8;
                    let c = [cx * 16, cy * 16, 255];
                    let ox = ix as i16 - 64;
                    let oy = iy as i16 - 64;
                    let oz = iz as i16 - 64;
                    let o = if ox * ox + oy * oy + oz * oz > 57 * 57 {
                        0
                    } else {
                        255
                    };
                    let v = Voxel::new(c, 255, 0, false, o);
                    obj.set_voxel(
                        v,
                        uvec3(
                            ix + BRICK_MAP_SIZE as u32 * 8,
                            iy + BRICK_MAP_SIZE as u32 * 8,
                            iz + BRICK_MAP_SIZE as u32 * 8,
                        ),
                    );
                }
            }
        }

        obj.process();
        obj
    }

    pub fn set_voxel(&mut self, voxel: Voxel, world_pos: UVec3) {
        let brick_pos = world_pos / 16;
        let local_pos = world_pos % 16;
        let brick_pos_1d = brick_pos.x as usize
            + brick_pos.y as usize * BRICK_MAP_SIZE
            + brick_pos.z as usize * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
        let brick_pool_idx = self.brick_map_cpu[brick_pos_1d] as usize;
        if brick_pool_idx == 0 {
            // Brick not yet allocated
            // Step 1: Find free brick
            let mut free_brick_idx = 0;
            for (i, flag) in self.brick_pool_flag_map.iter().enumerate() {
                if !flag.in_use() {
                    free_brick_idx = i + 1;
                    break;
                }
            }
            if free_brick_idx == 0 {
                error!("Failed to place voxel in world! No free bricks left :(");
                return;
                // todo!("Resize brick buffer?");
            }

            // Step 2: Allocate brick
            self.brick_map_cpu[brick_pos_1d] = free_brick_idx as u32;
            self.brick_pool_flag_map[free_brick_idx - 1].set_dirty(true);
            self.brick_pool_flag_map[free_brick_idx - 1].set_in_use(true);

            // Step 3: Set voxel in brick
            self.brick_pool_cpu[free_brick_idx as usize - 1].set_voxel(voxel, local_pos);
        } else {
            // Brick already allocated
            let brick = &mut self.brick_pool_cpu[brick_pool_idx - 1];
            brick.set_voxel(voxel, local_pos);
            self.brick_pool_flag_map[brick_pool_idx - 1].set_dirty(true);
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
        self.brick_pool_flag_map
            .iter_mut()
            .enumerate()
            .for_each(|(i, flag)| {
                if flag.dirty() {
                    if !self.brick_pool_cpu[i].is_empty() {
                        self.brick_pool.write(i, &[self.brick_pool_cpu[i]]);
                    } else {
                        // Brick is empty now, free it up
                        flag.set_in_use(false);
                    }
                    // self.brick_pool.write(i, &[self.brick_pool_cpu[i]]);
                    flag.set_dirty(false);
                }
            });

        // TODO: This is always uploaded, but that's very much overkill and bad for performance lol
        self.brick_map.write(0, &self.brick_map_cpu[..]);
    }
}
