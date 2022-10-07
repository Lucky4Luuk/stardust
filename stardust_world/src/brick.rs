use super::voxel::Voxel;

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

#[derive(Copy, Clone)]
pub struct BrickFlags(u8);

impl BrickFlags {
    pub fn empty() -> Self { Self(0) }

    fn set_bit(&mut self, bit: usize, val: bool) {
        if val { self.0 |= 1<<bit; } else { self.0 &= !(1<<bit); }
    }

    fn get_bit(&self, bit: usize) -> bool {
        ((self.0 >> bit) & 1) > 0
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.set_bit(0, dirty);
    }

    pub fn dirty(&self) -> bool {
        self.get_bit(0)
    }

    pub fn set_in_use(&mut self, in_use: bool) {
        self.set_bit(1, in_use);
    }

    pub fn in_use(&self) -> bool {
        self.get_bit(1)
    }
}
