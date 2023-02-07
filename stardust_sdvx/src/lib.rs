use serde::{Deserialize, Serialize};

use stardust_common::voxel::Voxel;
use stardust_common::math::*;

/// A model as represented in memory. Cannot be turned back into a RawModel.
#[derive(Debug)]
pub struct Model {
    voxels: Vec<Voxel>,
    positions: Vec<(UVec3, u32)>,
}

impl Model {
    pub fn from_voxels(combined: Vec<(Voxel, UVec3)>) -> Self {
        let mut voxels = Vec::new();
        let mut positions = Vec::new();
        let mut i = 0;
        for (vox, pos) in combined {
            voxels.push(vox);
            positions.push((uvec3(pos.x, pos.y, pos.z), i));
            i += 1;
        }
        Self {
            voxels,
            positions
        }
    }

    pub fn voxels(&self) -> impl Iterator<Item = (Voxel, &UVec3)> {
        self.positions.iter().map(|(pos, idx)| {
            (self.voxels[*idx as usize], pos)
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let raw: RawModel = ciborium::de::from_reader(bytes)?;

        let mut voxels = Vec::new();
        let mut positions = Vec::new();
        for raw_pos in raw.positions {
            let raw_vox = raw.voxels[raw_pos[3] as usize];
            voxels.push(Voxel(raw_vox));
            positions.push((uvec3(raw_pos[0], raw_pos[1], raw_pos[2]), raw_pos[3]));
        }

        Ok(Self {
            voxels,
            positions,
        })
    }

    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let raw_voxels: Vec<u32> = self.voxels.iter().map(|v| v.0).collect();
        let raw_positions: Vec<[u32; 4]> = self.positions.iter().map(|(pos, idx)| [pos.x,pos.y,pos.z,*idx]).collect();

        // TODO: Sort here, but keep them aligned!!!

        let raw = RawModel {
            voxels: raw_voxels,
            positions: raw_positions,
        };

        let mut bytes = Vec::new();
        ciborium::ser::into_writer(&raw, &mut bytes)?;
        Ok(bytes)
    }
}

#[derive(Serialize, Deserialize)]
struct RawModel {
    voxels: Vec<u32>,
    positions: Vec<[u32; 4]>,
}
