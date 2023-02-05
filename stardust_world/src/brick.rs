use stardust_common::math::*;
use stardust_common::voxel::Voxel;

const BRICK_SIZE: usize = 16*16*16 + 4;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Brick([Voxel; BRICK_SIZE]);

impl Brick {
    pub fn empty() -> Self {
        Self([Voxel::empty(); BRICK_SIZE])
    }

    pub fn full() -> Self {
        let mut data = [Voxel::new([255;3],255,0,false,255); BRICK_SIZE];
        for i in 0..4 { data[4096 + i] = Voxel(0); }
        Self(data)
    }

    pub fn is_empty(&self) -> bool {
        (self.0.iter().map(|v| v.opacity() as usize).sum::<usize>()) == 0
    }

    pub fn func<F: Fn(usize, usize, usize) -> Voxel>(f: F) -> Self {
        let mut brick = Self::empty();
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    brick.0[x+y*16+z*16*16] = f(x,y,z);
                }
            }
        }
        brick
    }

    pub fn set_voxel(&mut self, voxel: Voxel, pos: UVec3) {
        let i = pos.x as usize + pos.y as usize * 16 + pos.z as usize * 16 * 16;
        self.0[i] = voxel;
    }

    pub fn get_voxel(&self, pos: UVec3) -> &Voxel {
        let i = pos.x as usize + pos.y as usize * 16 + pos.z as usize * 16 * 16;
        &self.0[i]
    }
}
