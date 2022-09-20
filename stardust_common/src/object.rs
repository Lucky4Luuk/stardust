use crate::math::*;
use crate::shape::Shape;

pub enum Staticness {
    Full,
    Partial,
    None,
}

pub struct Object {
    pub voxel_data: Box<dyn Shape>,

    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,

    pub staticness: Staticness,
}

impl Object {
    pub fn from_shape(shape: Box<dyn Shape>) -> Self {
        Self {
            voxel_data: shape,

            pos: vec3(0.0, 0.0, 0.0),
            rot: Quat::IDENTITY,
            scale: vec3(1.0, 1.0, 1.0),

            staticness: Staticness::Full,
        }
    }
}
