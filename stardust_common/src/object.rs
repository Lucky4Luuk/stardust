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
