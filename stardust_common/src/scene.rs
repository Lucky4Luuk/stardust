use crate::shape::Shape;
use crate::object::Object;

pub struct Scene<S: Shape> {
    objects: Vec<Object<S>>,
}

impl<S: Shape> Scene<S> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }
}
