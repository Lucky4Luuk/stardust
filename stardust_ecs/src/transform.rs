use specs::prelude::*;

use stardust_common::math::*;

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct CompTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl CompTransform {
    pub fn new() -> Self {
        Self {
            position: vec3(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: vec3(0.0, 0.0, 0.0),
        }
    }
}
