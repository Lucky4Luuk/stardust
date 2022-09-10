use crate::math::*;
use crate::shape::Shape;

pub struct Object<S: Shape> {
    voxel_data: S,

    pos: Vec3,
    rot: Quat,
    scale: Vec3,
}
