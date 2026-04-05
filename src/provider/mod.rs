pub mod command;
pub mod repl;

use crate::engine::engine::Engine;

pub trait Provider {
    fn run(&self, engine: &mut Engine);
}
