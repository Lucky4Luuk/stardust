use crate::math::*;
use crate::shape::Shape;

pub struct Object {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    pub shape: Shape,
}
