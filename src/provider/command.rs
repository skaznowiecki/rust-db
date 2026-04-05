use crate::engine::engine::{Engine, ExecuteResult};
use super::Provider;

pub struct CommandProvider {
    pub sql: String,
}

impl Provider for CommandProvider {
    fn run(&self, engine: &mut Engine) {
        match engine.execute(&self.sql) {
            Ok(ExecuteResult::Message(msg)) => println!("{}", msg),
            Ok(ExecuteResult::DbChanged(name)) => {
                println!("Using database '{}'", name);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
