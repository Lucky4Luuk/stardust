use crate::math::*;
use crate::glsl::*;

pub trait Shape {
    /// Can be used to rename the default position argument from `p` to something else
    /// NOTE: Currently not used!
    fn glsl_arg_pos_name(&self) -> String { String::from("p") }
    /// Needs to return a GlslFunction with at least 1 argument named `p` for object position
    /// Also, the return type of said argument needs to be `GlslType::Vec3`!
    /// If this is not done, the generation code will mostlikely not work!
    fn glsl(&self) -> GlslFunction;
    /// Generates a call
    fn glsl_call(&self, func: &GlslFunction) -> String;
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

    fn glsl_call(&self, func: &GlslFunction) -> String {
        func.call_to_glsl().replace("r", &self.0.to_string())
    }
}

pub struct Cube(pub Vec3);
impl Shape for Cube {
    fn glsl(&self) -> GlslFunction {
        GlslFunction {
            return_type: GlslType::Float,
            name: String::from("sdCube"),
            args: vec![(String::from("p"), GlslType::Vec3), (String::from("b"), GlslType::Vec3)],

            code: String::from("vec3 q = abs(p) - b;\nreturn length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)), 0.0);"),
        }
    }

    fn glsl_call(&self, func: &GlslFunction) -> String {
        func.call_to_glsl().replace("b", &format!("vec3({}, {}, {})", self.0.x, self.0.y, self.0.z))
    }
}

pub struct Torus(pub [f32; 2]);
impl Shape for Torus {
    fn glsl(&self) -> GlslFunction {
        GlslFunction {
            return_type: GlslType::Float,
            name: String::from("sdTorus"),
            args: vec![(String::from("p"), GlslType::Vec3), (String::from("t"), GlslType::Vec2)],

            code: String::from("vec2 q = vec2(length(p.xz)-t.x,p.y);\nreturn length(q)-t.y;"),
        }
    }

    fn glsl_call(&self, func: &GlslFunction) -> String {
        func.call_to_glsl().replace("t", &format!("vec2({}, {})", self.0[0], self.0[1]))
    }
}
