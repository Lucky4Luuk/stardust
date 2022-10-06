use super::voxel::Voxel;

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
            voxels: [Voxel::new([255;3],255,false,255); 16*16*16]
        }
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
}
