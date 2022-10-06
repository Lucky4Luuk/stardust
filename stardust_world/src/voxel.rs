use stardust_common::math::*;

#[repr(C)]
pub struct VoxelWithPos((UVec4, Voxel));
impl VoxelWithPos {
    pub fn from_voxel(voxel: Voxel, pos: UVec3) -> Self {
        Self((uvec4(pos.x, pos.y, pos.z, 0), voxel))
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Voxel(u32);

impl Voxel {
    pub(crate) fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn empty() -> Self {
        Self::new([0;3],0,false,0)
    }

    pub fn new(rgb: [u8; 3], roughness: u8, metalic: bool, opacity: u8) -> Self {
        let r = (rgb[0] >> 3) as u16;
        let g = (rgb[1] >> 2) as u16;
        let b = (rgb[2] >> 3) as u16;
        let rgb = r | g << 5 | b << 11;
        let opacity_metalic: u8 = (opacity & 0b1111_1110) | (metalic as u8 & 0b0000_0001);
        let b: u32 = (rgb as u32) | ((roughness as u32) << 16) | ((opacity_metalic as u32) << 24);
        Self(b)
    }

    pub fn rgb(&self) -> [u8; 3] {
        let rgb_u16 = (self.0 & 0xFFFF) as u16;
        let r = rgb_u16 as u8;
        let g = (rgb_u16 >> 5) as u8;
        let b = (rgb_u16 >> 11) as u8;
        let r = r << 3;
        let g = g << 2;
        let b = b << 3;
        [r as u8,g as u8,b as u8]
    }

    pub fn roughness(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn opacity(&self) -> u8 {
        (self.0 >> 24) as u8 & 0b1111_1110
    }

    pub fn metalic(&self) -> bool {
        ((self.0 >> 24) as u8 & 0b0000_0001) != 0
    }
}
