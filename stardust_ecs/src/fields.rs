use std::collections::BTreeMap;
use std::sync::Arc;
use thiserror::Error;
use specs::prelude::*;
use stardust_world::GpuModel;

#[derive(Debug, Error)]
pub enum FieldError {
    #[error("Generic FieldError")]
    Generic,
    #[error("Field does not exist!")]
    FieldDoesNotExist,
    #[error("Field has wrong value!")]
    FieldHasWrongValue,
    #[error("Field value unsupported for this operation!")]
    FieldValueUnsupported,
}

pub enum ValueOwned {
    // Primitives
    String(String),
    Bool(bool),

    PrimF32(f32),

    PrimU8(u8),
    PrimU16(u16),
    PrimU32(u32),
    PrimU64(u64),

    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Vec4(f32, f32, f32, f32),

    // Complex values
    ModelReference(Option<Arc<GpuModel>>),
}

pub enum Value<'a> {
    // Primitives
    String(&'a mut String),
    Bool(&'a mut bool),

    PrimF32(&'a mut f32),

    PrimU8(&'a mut u8),
    PrimU16(&'a mut u16),
    PrimU32(&'a mut u32),
    PrimU64(&'a mut u64),

    Vec2(&'a mut f32, &'a mut f32),
    Vec3(&'a mut f32, &'a mut f32, &'a mut f32),
    Vec4(&'a mut f32, &'a mut f32, &'a mut f32, &'a mut f32),

    // Complex values
    ModelReference(&'a mut Option<Arc<GpuModel>>),
}

impl<'a> Value<'a> {
    pub fn set_from_owned(&mut self, owned: ValueOwned) -> Result<(), FieldError> {
        match self {
            Self::String(v_ref) => if let ValueOwned::String(v) = owned {
                **v_ref = v;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            Self::Bool(v_ref) => if let ValueOwned::Bool(v) = owned {
                **v_ref = v;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            Self::PrimF32(v_ref) => if let ValueOwned::PrimF32(v) = owned {
                **v_ref = v;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            Self::PrimU8(v_ref) => if let ValueOwned::PrimU8(v) = owned {
                **v_ref = v;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            Self::PrimU16(v_ref) => if let ValueOwned::PrimU16(v) = owned {
                **v_ref = v;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            Self::PrimU32(v_ref) => if let ValueOwned::PrimU32(v) = owned {
                **v_ref = v;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            Self::PrimU64(v_ref) => if let ValueOwned::PrimU64(v) = owned {
                **v_ref = v;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            Self::Vec2(v1_ref, v2_ref) => if let ValueOwned::Vec2(v1, v2) = owned {
                **v1_ref = v1;
                **v2_ref = v2;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            Self::Vec3(v1_ref, v2_ref, v3_ref) => if let ValueOwned::Vec3(v1, v2, v3) = owned {
                **v1_ref = v1;
                **v2_ref = v2;
                **v3_ref = v3;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            Self::Vec4(v1_ref, v2_ref, v3_ref, v4_ref) => if let ValueOwned::Vec4(v1, v2, v3, v4) = owned {
                **v1_ref = v1;
                **v2_ref = v2;
                **v3_ref = v3;
                **v4_ref = v4;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            Self::ModelReference(v_ref) => if let ValueOwned::ModelReference(v) = owned {
                **v_ref = v;
                Ok(())
            } else {
                Err(FieldError::FieldHasWrongValue)
            },
            _ => Err(FieldError::FieldValueUnsupported),
        }
    }
}

pub trait EngineComponentWritable {
    fn write(&self, world: &mut World, entity: Entity);
}

impl<T: Component + Clone> EngineComponentWritable for T {
    fn write(&self, world: &mut World, entity: Entity) {
        let mut storage = world.write_storage::<Self>();
        if let Some(comp) = storage.get_mut(entity) {
            *comp = self.clone();
        } else {
            storage.insert(entity, self.clone()).expect("Failed to add component!");
        }
    }
}

pub trait EngineComponent: EngineComponentWritable {
    fn fields(&mut self) -> BTreeMap<String, (bool, Value)>;
    fn set_field(&mut self, name: &str, value: ValueOwned) -> Result<(), FieldError> {
        let mut fields = self.fields();
        if let Some((_, value_ref)) = fields.get_mut(name) {
            value_ref.set_from_owned(value)
        } else {
            Err(FieldError::FieldDoesNotExist)
        }
    }
}
