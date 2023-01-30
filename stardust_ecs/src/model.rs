use specs::prelude::*;

use std::collections::BTreeMap;

use stardust_common::math::*;

use crate::Value;

#[derive(Debug, Component, Clone)]
#[storage(DenseVecStorage)]
pub struct CompModel {
    pub vox_pos: UVec3,
}

impl CompModel {
    pub fn new() -> Self {
        Self {
            vox_pos: uvec3(0,0,0),
        }
    }

    pub fn fields(&mut self) -> BTreeMap<String, Value> {
        let mut map = BTreeMap::new();
        map
    }

    /// Returns true if the new location is different to the current position
    /// This is useful for moving the model in the voxel world, as that is not
    /// a cheap operation!
    pub fn update_voxel_position(&mut self, new_vox_pos: UVec3) -> bool {
        if self.vox_pos == new_vox_pos {
            return false;
        }

        self.vox_pos = new_vox_pos;
        true
    }
}
