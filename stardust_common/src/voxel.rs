pub trait IsVoxel {
    fn generic_new(rgb: [u8; 3], roughness: u8, emissive: u8, metallic: bool, opacity: u8) -> Self;
}
