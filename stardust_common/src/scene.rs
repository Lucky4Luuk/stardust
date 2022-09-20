use crate::glsl::*;
use crate::object::*;

pub struct Scene {
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn new() -> Self {
        Self::with_objects(Vec::new())
    }

    pub fn with_objects(objects: Vec<Object>) -> Self {
        Self {
            objects: objects,
        }
    }

    pub fn get_glsl(&self) -> Vec<GlslFunction> {
        let mut funcs = Vec::new();

        for obj in &self.objects {
            funcs.push(obj.voxel_data.glsl());
        }

        // Generate map function
        let mut code = String::new();
        code.push_str("float d = T_MAX;\n");
        for obj in &self.objects {
            match &obj.staticness {
                Staticness::Full => code.push_str(&format!("d = min(d, {});\n", obj.voxel_data.glsl_call(&obj.voxel_data.glsl()))),
                _ => todo!(),
            }
        }
        code.push_str("return d;\n");

        funcs.push(GlslFunction {
            return_type: GlslType::Float,
            name: String::from("map"),
            args: vec![(String::from("p"), GlslType::Vec3)],

            code: code,
        });

        funcs
    }
}
