use specs::prelude::*;
use ecs_derive::EngineComponent;

use std::sync::Arc;

use stardust_common::math::*;
use stardust_world::GpuModel;

use crate::{Value, ValueOwned, FieldError, FieldMap};

#[derive(Component, Clone, EngineComponent)]
#[storage(DenseVecStorage)]
pub struct CompModel {
    pub prev_vox_pos: UVec3,
    pub vox_pos: UVec3,
    #[visible]
    pub dirty: bool,

    #[editable("Model")]
    pub model_ref: Option<Arc<GpuModel>>,
    pub next_model: Option<Arc<GpuModel>>,
}

impl CompModel {
    pub fn new() -> Self {
        Self {
            prev_vox_pos: uvec3(0,0,0),
            vox_pos: uvec3(0,0,0),
            dirty: false,

            model_ref: None,
            next_model: None,
        }
    }

    /// Returns true if the new location is different to the current position
    pub(crate) fn update_voxel_position(&mut self, new_vox_pos: UVec3) {
        if self.vox_pos == new_vox_pos { return; }

        self.prev_vox_pos = self.vox_pos;
        self.vox_pos = new_vox_pos;
        self.dirty = true;
    }

    pub(crate) fn update_model_ref(&mut self) {
        self.model_ref = self.next_model.clone();
        self.next_model = None;
    }
}

impl crate::EngineComponentSetField<CompModel> for CompModel {
    fn set_field(&mut self, name: &str, value: ValueOwned) -> Result<(), FieldError> {
        use crate::EngineComponentGetField;
        let mut fields = self.fields();
        match name {
            // Special case for this component
            "model" | "Model" | "model_ref" => if let ValueOwned::ModelReference(model_owned) = value {
                self.next_model = model_owned;
                self.dirty = true;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            _ => if let Some((_, value_ref)) = fields.get_mut(name) {
                value_ref.set_from_owned(value)
            } else {
                Err(FieldError::FieldDoesNotExist)
            },
        }
    }
}
