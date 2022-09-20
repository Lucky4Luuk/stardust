// Testing file

use stardust_common::shape::*;
use stardust_common::object::*;
use stardust_common::math::*;
use stardust_common::scene::*;

fn main() {
    let x = Object::from_shape(Box::new(Sphere(5.)));
    let y = Object::from_shape(Box::new(Cube(vec3(3.,2.,1.))));
    let scene = Scene::with_objects(vec![x, y]);
    for func in scene.get_glsl() {
        println!("{:#?}\n", func);
        // println!("{}\n", func.to_glsl());
    }
}
