use specs::prelude::*;
use ecs_derive::EngineComponent;

use stardust_common::math::*;

use crate::{Value, FieldMap};

#[derive(Debug, Component, Clone, EngineComponent)]
#[storage(VecStorage)]
pub struct CompTransform {
    pub position: Vec3,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub rotation_w: f32,
    pub scale: Vec3,
}

impl CompTransform {
    pub fn new() -> Self {
        let r = Quat::IDENTITY.to_array();
        Self {
            position: vec3(0.0, 0.0, 0.0),
            rotation_x: r[0],
            rotation_y: r[1],
            rotation_z: r[2],
            rotation_w: r[3],
            scale: vec3(0.0, 0.0, 0.0),
        }
    }
}
