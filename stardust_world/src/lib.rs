#[macro_use]
extern crate log;

use rayon::prelude::*;
use foxtail::prelude::*;

use stardust_common::math::*;
use stardust_common::voxel::Voxel;

pub mod usage_flags;
pub mod layer0;
pub mod brick;

use usage_flags::*;
use layer0::*;
use brick::*;

pub const BRICK_POOL_SIZE: usize = 32768;
pub const LAYER0_POOL_SIZE: usize = 8192;
const BRICK_MAP_SIZE: usize = 64;

pub struct World {
    brick_pool: FixedSizeBuffer<Brick>,
    layer0_pool: FixedSizeBuffer<Layer0>,
    layer0_map: FixedSizeBuffer<u32>,

    brick_pool_cpu: Box<[Brick]>,
    brick_pool_flag_map: Box<[UsageFlags]>,

    layer0_pool_cpu: Box<[Layer0]>,
    layer0_pool_flag_map: Box<[UsageFlags]>,

    layer0_map_cpu: Box<[u32]>,

    pub bricks_used: usize,
    pub layer0_used: usize,
}

impl World {
    pub fn new(ctx: &Context) -> Self {
        debug!("Creating new world...");
        let brick_pool = FixedSizeBuffer::new(ctx, BRICK_POOL_SIZE);
        debug!("Brick pool created!");
        let layer0_pool = FixedSizeBuffer::new(ctx, LAYER0_POOL_SIZE);
        debug!("Layer0 pool created!");
        let layer0_map = FixedSizeBuffer::new(ctx, BRICK_MAP_SIZE * BRICK_MAP_SIZE * BRICK_MAP_SIZE);
        debug!("Brick map created!");

        let brick_pool_cpu: Box<[Brick]> = vec![Brick::empty(); BRICK_POOL_SIZE].into_boxed_slice();
        let layer0_pool_cpu: Box<[Layer0]> = vec![Layer0::empty(); LAYER0_POOL_SIZE].into_boxed_slice();
        let layer0_map_cpu: Box<[u32]> = vec![0u32; BRICK_MAP_SIZE * BRICK_MAP_SIZE * BRICK_MAP_SIZE].into_boxed_slice();

        let mut flag = UsageFlags::empty();
        flag.set_dirty(true);
        let brick_pool_flag_map = vec![flag; BRICK_POOL_SIZE].into_boxed_slice();
        let layer0_pool_flag_map = vec![flag; LAYER0_POOL_SIZE].into_boxed_slice();

        let mut obj = Self {
            brick_pool: brick_pool,
            layer0_pool: layer0_pool,
            layer0_map: layer0_map,

            brick_pool_cpu: brick_pool_cpu,
            brick_pool_flag_map: brick_pool_flag_map,

            layer0_pool_cpu: layer0_pool_cpu,
            layer0_pool_flag_map: layer0_pool_flag_map,

            layer0_map_cpu: layer0_map_cpu,

            bricks_used: 0,
            layer0_used: 0,
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
                            ix + 1024,
                            iy + 1024,
                            iz + 1024,
                        ),
                    );
                }
            }
        }

        obj.process();
        obj
    }

    pub fn set_voxel(&mut self, voxel: Voxel, world_pos: UVec3) {
        puffin::profile_function!();
        let layer0_pos = world_pos / 16 / 16;

        let layer0_pos_1d = layer0_pos.x as usize
            + layer0_pos.y as usize * BRICK_MAP_SIZE
            + layer0_pos.z as usize * BRICK_MAP_SIZE * BRICK_MAP_SIZE;

        let layer0_pool_idx = self.layer0_map_cpu[layer0_pos_1d] as usize;
        if layer0_pool_idx == 0 {
            // Layer0Node not yet allocated
            // Step 1: Find free Layer0Node
            let mut free_layer0_idx = 0;
            for (i, flag) in self.layer0_pool_flag_map.iter().enumerate() {
                if !flag.in_use() {
                    free_layer0_idx = i + 1;
                    break;
                }
            }
            if free_layer0_idx == 0 {
                error!("Failed to place voxel in world! No free Layer0Nodes left :(");
                return;
                // todo!("Resize layer0 buffer?");
            }

            // Step 2: Allocate Layer0Node
            self.layer0_map_cpu[layer0_pos_1d] = free_layer0_idx as u32;
            self.layer0_pool_flag_map[free_layer0_idx - 1].set_dirty(true);
            self.layer0_pool_flag_map[free_layer0_idx - 1].set_in_use(true);
            self.layer0_used += 1;

            self.set_voxel_in_layer0(voxel, world_pos, free_layer0_idx - 1);
        } else {
            // Layer0 already allocated
            self.set_voxel_in_layer0(voxel, world_pos, layer0_pool_idx - 1);
        }
    }

    /// Assumes the Layer0Node at layer0_idx to already be allocated.
    fn set_voxel_in_layer0(&mut self, voxel: Voxel, world_pos: UVec3, layer0_idx: usize) {
        puffin::profile_function!();
        let brick_pos = (world_pos / 16) % 16;

        let brick_pos_1d = brick_pos.x as usize
            + brick_pos.y as usize * 16
            + brick_pos.z as usize * 16 * 16;

        let layer0 = &mut self.layer0_pool_cpu[layer0_idx];
        let brick_pool_idx = layer0.brick_indices[brick_pos_1d] as usize;
        if brick_pool_idx == 0 {
            // Brick not yet allocated
            // Step 1: Find free Brick
            let mut free_brick_idx = 0;
            for (i, flag) in self.brick_pool_flag_map.iter().enumerate() {
                if !flag.in_use() {
                    free_brick_idx = i + 1;
                    break;
                }
            }
            if free_brick_idx == 0 {
                error!("Failed to place voxel in world! No free Bricks left :(");
                return;
                // todo!("Resize brick buffer?");
            }

            // Step 2: Allocate Brick
            layer0.brick_indices[brick_pos_1d] = free_brick_idx as u32;
            self.brick_pool_flag_map[free_brick_idx - 1].set_dirty(true);
            self.brick_pool_flag_map[free_brick_idx - 1].set_in_use(true);
            self.bricks_used += 1;

            self.set_voxel_in_brick(voxel, world_pos, free_brick_idx - 1);
        } else {
            // Brick already allocated
            self.set_voxel_in_brick(voxel, world_pos, brick_pool_idx - 1);
        }
    }

    /// Assumes the Brick at brick_idx to already be allocated
    fn set_voxel_in_brick(&mut self, voxel: Voxel, world_pos: UVec3, brick_idx: usize) {
        puffin::profile_function!();
        let local_pos = world_pos % 16;
        let brick = &mut self.brick_pool_cpu[brick_idx];
        brick.set_voxel(voxel, local_pos);
        self.brick_pool_flag_map[brick_idx].set_dirty(true);
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

    pub fn process(&mut self) {
        puffin::profile_function!();
        let to_write_layer0: Vec<usize> = self.layer0_pool_flag_map
            .par_iter_mut()
            .enumerate()
            .map(|(i, flag)| {
                if flag.dirty() {
                    flag.set_dirty(false);
                    i + 1
                } else {
                    0
                }
            }).filter(|i| *i > 0).collect();

        let to_write_brick: Vec<usize> = self.brick_pool_flag_map
            .par_iter_mut()
            .enumerate()
            .map(|(i, flag)| {
                let mut ret = 0;
                if flag.dirty() {
                    if !self.brick_pool_cpu[i].is_empty() {
                        ret = i + 1;
                    } else if flag.in_use() {
                        // Brick is empty now, free it up
                        flag.set_in_use(false);
                    }
                    flag.set_dirty(false);
                }
                ret
            }).filter(|i| *i > 0).collect();

        to_write_layer0.into_iter().for_each(|i| {
            let i = i - 1;
            self.layer0_pool.write(i, &[self.layer0_pool_cpu[i]]);
        });

        // to_write_brick.into_iter().for_each(|i| {
        //     let i = i - 1;
        //     self.brick_pool.write(i, &[self.brick_pool_cpu[i]]);
        // });
        let to_write_brick_data = to_write_brick.into_iter().map(|i| {
            let i = i - 1;
            (i, &self.brick_pool_cpu[i])
        });
        self.brick_pool.write_slice(to_write_brick_data);

        // TODO: This is always uploaded, but that's very much overkill and bad for performance scaling lol
        self.layer0_map.write(0, &self.layer0_map_cpu[..]);
    }
}
