use crate::mesh::Mesh;
use crate::object::Object;

pub struct Scene<M: Mesh> {
    objects: Vec<Object<M>>,
}
