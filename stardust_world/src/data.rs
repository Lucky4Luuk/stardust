use foxtail::prelude::*;

use stardust_common::math::*;
use stardust_common::voxel::Voxel;

use stardust_sdvx::Model;

/// MUST BE CREATED IN THE MAIN THREAD
pub struct GpuModel {
    pub(crate) vox_buf: FixedSizeBuffer<[u32; 4]>, // xyz = pos, w = voxel
    pub(crate) voxels: usize,
}

impl GpuModel {
    /// MUST BE RUN FROM THE MAIN THREAD
    pub fn from_voxels(ctx: &Context, voxels: &Vec<(Voxel, UVec3)>) -> Self {
        let mut voxel_data = Vec::new();
        for (vox, pos) in voxels {
            voxel_data.push([pos.x + 1024, pos.y + 1024, pos.z + 1024, vox.0]);
        }

        let vox_buf = FixedSizeBuffer::new(ctx, voxels.len());
        vox_buf.write(0, &voxel_data);

        trace!("wahoo: {}", voxels.len());

        Self {
            vox_buf,
            voxels: voxels.len(),
        }
    }

    /// MUST BE RUN FROM THE MAIN THREAD
    pub fn from_model(ctx: &Context, model: &Model) -> Self {
        Self::from_voxels(ctx, &model.voxels().map(|(v,p)| (v, *p)).collect())
    }

    pub fn voxel_count(&self) -> usize {
        self.voxels
    }
}
