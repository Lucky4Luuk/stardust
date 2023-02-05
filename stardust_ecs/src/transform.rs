use specs::prelude::*;

use stardust_common::math::*;

use crate::{Value, FieldMap};

#[derive(Debug, Component, Clone)]
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

impl crate::EngineComponent for CompTransform {
    fn fields(&mut self) -> FieldMap {
        let mut map = FieldMap::new();
        map.insert(String::from("pos"), (true, Value::Vec3(&mut self.position.x, &mut self.position.y, &mut self.position.z)));
        map.insert(String::from("rot"), (true, Value::Vec4(&mut self.rotation_x, &mut self.rotation_y, &mut self.rotation_z, &mut self.rotation_w)));
        map.insert(String::from("scl"), (true, Value::Vec3(&mut self.scale.x, &mut self.scale.y, &mut self.scale.z)));
        map
    }
}
