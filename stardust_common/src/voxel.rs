use crate::math::*;

#[repr(C)]
pub struct VoxelWithPos((UVec4, Voxel));
impl VoxelWithPos {
    pub fn from_voxel(voxel: Voxel, pos: UVec3) -> Self {
        Self((uvec4(pos.x, pos.y, pos.z, 0), voxel))
    }
}

/// Format (bits):
/// [0-15]  - rgb565
/// [16-19] - roughness
/// [20-23] - emissive
/// [24]    - metallic
/// [25-31] - opacity
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Voxel(pub u32);

impl Voxel {
    /// Creates a new voxel. Opacity = 0 does not mean the voxel isn't there! A voxel is only
    /// seen as "empty" if everything is set to 0. Recommended function for this is `Self::empty()`
    pub fn new(rgb: [u8; 3], roughness: u8, emissive: u8, metallic: bool, opacity: u8) -> Self {
        let r = (rgb[0] >> 3) as u16;
        let g = (rgb[1] >> 2) as u16;
        let b = (rgb[2] >> 3) as u16;
        let rgb = r | g << 5 | b << 11;
        let roughness_emissive = (roughness >> 4) | (emissive >> 4 << 4);
        let opacity_metalic: u8 = (opacity & 0b1111_1110) | (metallic as u8 & 0b0000_0001);
        let b: u32 = (rgb as u32) | ((roughness_emissive as u32) << 16) | ((opacity_metalic as u32) << 24);
        Self(b)
    }

    pub fn empty() -> Self {
        Self(0)
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
        ((self.0 >> 16) as u8) << 4
    }

    pub fn emissive(&self) -> u8 {
        ((self.0 >> 20) as u8) << 4
    }

    pub fn opacity(&self) -> u8 {
        (self.0 >> 24) as u8 & 0b1111_1110
    }

    pub fn metallic(&self) -> bool {
        ((self.0 >> 24) as u8 & 0b0000_0001) != 0
    }
}
