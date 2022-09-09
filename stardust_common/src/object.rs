use crate::math::*;
use crate::mesh::Mesh;

pub struct Object<M: Mesh> {
    mesh: M,

    pos: Vec3,
    rot: Quat,
    scale: Vec3,
}
