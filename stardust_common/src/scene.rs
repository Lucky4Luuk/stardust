use crate::object::Object;

pub struct Scene {
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }
}
