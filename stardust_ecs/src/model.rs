use specs::prelude::*;

use std::collections::BTreeMap;
use std::sync::Arc;

use stardust_common::math::*;
use stardust_world::GpuModel;

use crate::Value;

#[derive(Component, Clone)]
#[storage(DenseVecStorage)]
pub struct CompModel {
    pub prev_vox_pos: UVec3,
    pub vox_pos: UVec3,
    pub dirty: bool,

    pub model_ref: Option<Arc<GpuModel>>,
}

impl CompModel {
    pub fn new() -> Self {
        Self {
            prev_vox_pos: uvec3(0,0,0),
            vox_pos: uvec3(0,0,0),
            dirty: false,

            model_ref: None,
        }
    }

    pub fn fields(&mut self) -> BTreeMap<String, (bool, Value)> {
        let mut map = BTreeMap::new();
        map.insert("Model".to_string(), (true, Value::ModelReference(&mut self.model_ref)));
        map.insert("Dirty".to_string(), (false, Value::Bool(&mut self.dirty)));
        map
    }

    /// Returns true if the new location is different to the current position
    /// This is useful for moving the model in the voxel world, as that is not
    /// a cheap operation!
    pub fn update_voxel_position(&mut self, new_vox_pos: UVec3) -> bool {
        if self.vox_pos == new_vox_pos {
            return false;
        }

        self.prev_vox_pos = self.vox_pos;
        self.vox_pos = new_vox_pos;
        self.dirty = true;
        true
    }
}
