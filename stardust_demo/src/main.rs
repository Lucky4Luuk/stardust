#[macro_use] extern crate log;
use stardust::*;

pub struct Demo {

}

impl Demo {
    fn new() -> Self {
        Self {
            
        }
    }
}

impl App for Demo {
    fn update(&self) {}
    fn render(&self) {}
}

fn main() {
    stardust::run(Demo::new())
}
