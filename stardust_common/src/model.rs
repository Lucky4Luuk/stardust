use crate::voxel::IsVoxel;

pub trait IsModel<V: IsVoxel> {
    fn voxel_bounds(&self) -> (usize, usize, usize);
    /// Function loops over the voxel model bounds, and for each voxel, the model must return
    /// a voxel
    fn map_voxels(&self, f: dyn Fn(usize, usize, usize) -> V);
}
