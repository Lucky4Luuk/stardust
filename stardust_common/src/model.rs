use crate::math::*;
use crate::voxel::IsVoxel;

use anyhow::Result;

pub trait IsModel<V: IsVoxel> {
    fn from_path(path: &str) -> Result<Self> where Self: Sized;
    /// Returns the bounds of the model
    fn voxel_bounds(&self) -> (IVec3, IVec3);
    /// Get voxel from model
    fn get_voxel(&self, pos: IVec3) -> V;
}
