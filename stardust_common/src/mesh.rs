use crate::math::*;
use crate::camera::Camera;

pub trait Mesh {
    fn draw(&self, camera: &Camera, model: Mat4);
}
