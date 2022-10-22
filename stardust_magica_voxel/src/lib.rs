use stardust_common::{
    voxel::IsVoxel,
    model::IsModel,
    math::*,
};

use dot_vox::*;

use anyhow::Result;

#[derive(Debug, Clone)]
struct MagicaVoxelError(String);

impl std::fmt::Display for MagicaVoxelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for MagicaVoxelError {}

pub struct MagicaVoxelModel {
    model: DotVoxData,
}

impl<V: IsVoxel> IsModel<V> for MagicaVoxelModel {
    fn from_path(path: &str) -> Result<Self> {
        let model = dot_vox::load(path).map_err(|e| MagicaVoxelError(e.to_string()))?;

        Ok(Self {
            model: model,
        })
    }

    fn voxel_bounds(&self) -> (IVec3, IVec3) {
        let mut min = ivec3(0,0,0);
        let mut max = ivec3(0,0,0);

        (min, max)
    }

    fn get_voxel(&self, pos: IVec3) -> V {
        todo!();
    }
}
