pub mod client;
pub mod command;
pub mod repl;
pub mod server;

use crate::engine::engine::Engine;

pub trait Provider {
    fn run(&self, engine: &mut Engine);
}
