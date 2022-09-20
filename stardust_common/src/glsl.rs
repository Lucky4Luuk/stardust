#[derive(Debug)]
pub enum GlslType {
    Void,

    Float,
    Int,
    Uint,
    Bool,

    Vec2,
    Vec3,
    Vec4,

    IVec2,
    IVec3,
    IVec4,

    UVec2,
    UVec3,
    UVec4,

    BVec2,
    BVec3,
    BVec4,
}

impl GlslType {
    pub fn to_glsl(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

#[derive(Debug)]
pub struct GlslFunction {
    pub return_type: GlslType,
    pub name: String,
    pub args: Vec<(String, GlslType)>,

    pub code: String,
}

impl GlslFunction {
    pub fn to_glsl(&self) -> String {
        format!(
            "{} {}({}) {{ {} }}",
            self.return_type.to_glsl(),
            self.name,
            self.args.iter().map(|(n,t)| format!("{} {}", t.to_glsl(), n)).collect::<Vec<String>>().join(", "),
            self.code,
        )
    }

    pub fn call_to_glsl<F: Fn(&str) -> String>(&self, arg_map: F) -> String {
        format!(
            "{}({})",
            self.name,
            self.args.iter().map(|(n,_t)| arg_map(n.as_str())).collect::<Vec<String>>().join(", "),
        )
    }
}
