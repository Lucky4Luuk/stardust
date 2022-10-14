use crate::math::*;
use crate::voxel::IsVoxel;
use crate::model::IsModel;

pub struct Object<V: IsVoxel> {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    pub model: Box<dyn IsModel<V>>,
}
