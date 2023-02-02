use stardust_common::{
    voxel::Voxel as SDVoxel,
    math::*,
};
use stardust_sdvx::Model;

use dot_vox::*;

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
struct MagicaVoxelError(String);

impl std::fmt::Display for MagicaVoxelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

struct MagicaVoxelMaterial {
    rgb: [u8; 3],
    opacity: u8,
}

pub struct MagicaVoxelModel {
    voxels: Vec<(IVec3, u8)>,
    min: IVec3,
    max: IVec3,

    palette: Vec<MagicaVoxelMaterial>,
}

impl MagicaVoxelModel {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let model = dot_vox::load_bytes(bytes).map_err(|e| MagicaVoxelError(e.to_string()))?;

        let mut voxels = Vec::new();
        let mut min = ivec3(0,0,0);
        let mut max = ivec3(0,0,0);

        fn handle_node(node: &SceneNode, model: &DotVoxData, min: &mut IVec3, max: &mut IVec3, voxels: &mut Vec<(IVec3, u8)>) {
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
                                    let wpos = pos + vpos;
                                    *min = min.min(wpos);
                                    *max = max.max(wpos);
                                    voxels.push((wpos, voxel.i));
                                }
                            }
                        },
                        SceneNode::Group { attributes: _, children } => {
                            for child in children {
                                let child_node = &model.scenes[*child as usize];
                                handle_node(child_node, model, min, max, voxels);
                            }
                        }
                        _ => panic!("Wtf? {:?}", child_node),
                    }
                },
                _ => {}, // Don't handle this
            }
        }

        for node in &model.scenes {
            handle_node(node, &model, &mut min, &mut max, &mut voxels);
        }

        let mut palette = Vec::new();
        for (_i, color) in model.palette.iter().enumerate() {
            palette.push(MagicaVoxelMaterial {
                rgb: [color.r, color.g, color.b],
                opacity: color.a,
            });
        }

        Ok(Self {
            voxels: voxels,
            min: min,
            max: max,

            palette,
        })
    }

    pub fn voxel_bounds(&self) -> (IVec3, IVec3) {
        (self.min, self.max)
    }

    pub fn get_voxel(&self, pos: IVec3) -> SDVoxel {
        todo!();
    }

    pub fn to_sdvx(self) -> Model {
        let mut voxels = Vec::new();
        let (min, _max) = self.voxel_bounds();
        for (vpos, pal_idx) in self.voxels {
            let wpos_raw = vpos + min;
            let wpos = uvec3(wpos_raw.x as u32, wpos_raw.z as u32, wpos_raw.y as u32);
            let rgb = self.palette[pal_idx as usize].rgb;
            let sdvoxel = SDVoxel::new(rgb, 255, 0, false, 255);
            voxels.push((sdvoxel, wpos));
        }
        Model::from_voxels(voxels)
    }
}
