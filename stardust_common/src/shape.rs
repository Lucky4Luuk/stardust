use crate::math::*;
use crate::glsl::*;

pub trait Shape {
    /// Needs to return a GlslFunction with at least 1 argument named `p` for object position
    /// If this is not done, the generation code will mostlikely not work!
    fn glsl(&self) -> GlslFunction;
}

pub struct Sphere(pub f32);
impl Shape for Sphere {
    fn glsl(&self) -> GlslFunction {
        GlslFunction {
            return_type: GlslType::Float,
            name: String::from("sdSphere"),
            args: vec![(String::from("p"), GlslType::Vec3), (String::from("r"), GlslType::Float)],

            code: String::from("return length(p) - r;"),
        }
    }
}
