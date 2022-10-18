use stardust_common::math::*;

use super::voxel::Voxel;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Brick {
    voxels: [Voxel; 16*16*16],
}

impl Brick {
    pub fn empty() -> Self {
        Self {
            voxels: [Voxel::empty(); 16*16*16]
        }
    }

    pub fn full() -> Self {
        Self {
            voxels: [Voxel::new([255;3],255,0,false,255); 16*16*16]
        }
    }

    pub fn is_empty(&self) -> bool {
        (self.voxels.iter().map(|v| v.opacity() as usize).sum::<usize>()) == 0
    }

    pub fn func<F: Fn(usize, usize, usize) -> Voxel>(f: F) -> Self {
        let mut brick = Self::empty();
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    brick.voxels[x+y*16+z*16*16] = f(x,y,z);
                }
            }
        }
        brick
    }

    pub fn set_voxel(&mut self, voxel: Voxel, pos: UVec3) {
        let i = pos.x as usize + pos.y as usize * 16 + pos.z as usize * 16 * 16;
        self.voxels[i] = voxel;
    }
}
