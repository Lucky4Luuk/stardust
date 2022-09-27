#[macro_use] extern crate log;
use stardust::{*, rendering::*};

pub struct Demo {
    mesh: mesh::Mesh,
}

impl Demo {
    fn new(ctx: &Context) -> Self {
        ctx.set_window_title("Stardust demo");
        trace!("Demo created!");
        let mesh = mesh::Mesh::quad(&ctx);
        Self {
            mesh: mesh,
        }
    }
}

impl App for Demo {
    fn update(&self, ctx: &Context) {}
    fn render(&self, ctx: &Context) {}
}

fn main() {
    stardust::run(|ctx| Demo::new(ctx))
}
