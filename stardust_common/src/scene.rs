use crate::voxel::IsVoxel;
use crate::object::Object;

pub struct Scene<V: IsVoxel> {
    pub objects: Vec<Object<V>>,
}

impl<V: IsVoxel> Scene<V> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }
}
