// Testing file

use stardust_common::shape::*;

fn main() {
    let x = Sphere(5.);
    println!("`{}`", x.glsl().to_glsl());
}
