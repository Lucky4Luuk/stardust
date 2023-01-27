use stardust_common::{
    voxel::Voxel as SDVoxel,
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
    voxels: Vec<(IVec3, Voxel)>,
    min: IVec3,
    max: IVec3,
}

impl MagicaVoxelModel {
    pub fn from_path(path: &str) -> Result<Self> {
        let model = dot_vox::load(path).map_err(|e| MagicaVoxelError(e.to_string()))?;

        let mut voxels = Vec::new();
        let mut min = ivec3(0,0,0);
        let mut max = ivec3(0,0,0);

        for node in &model.scenes {
            match node {
                SceneNode::Transform { attributes: _, frames, child, layer_id: _ } => {
                    let child_node = &model.scenes[*child as usize];
                    let pos = if let Some(frame) = frames.get(0) {
                        frame.position().map(|p| ivec3(p.x,p.y,p.z)).unwrap_or(ivec3(0,0,0))
                    } else { ivec3(0,0,0) };
                    match child_node {
                        SceneNode::Shape { attributes: _, models } => {
                            for smodel in models {
                                let vmodel = &model.models[smodel.model_id as usize];
                                for voxel in &vmodel.voxels {
                                    let vpos = ivec3(voxel.x.into(),voxel.y.into(),voxel.z.into());
                                    min = min.min(vpos);
                                    max = max.max(vpos);
                                    voxels.push((pos + vpos, *voxel));
                                }
                            }
                        },
                        _ => panic!("Wtf?"),
                    }
                },
                _ => {}, // Don't handle this
            }
        }

        Ok(Self {
            voxels: voxels,
            min: min,
            max: max,
        })
    }

    pub fn voxel_bounds(&self) -> (IVec3, IVec3) {
        (self.min, self.max)
    }

    pub fn get_voxel(&self, pos: IVec3) -> SDVoxel {
        todo!();
    }
}
