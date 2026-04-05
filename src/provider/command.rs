use crate::engine::engine::{Engine, ExecuteResult};
use super::client;
use super::Provider;

pub enum CommandMode {
    Local,
    Remote(u16),
}

pub struct CommandProvider {
    pub sql: String,
    pub mode: CommandMode,
}

impl Provider for CommandProvider {
    fn run(&self, engine: &mut Engine) {
        match &self.mode {
            CommandMode::Local => match engine.execute(&self.sql) {
                Ok(ExecuteResult::Message(msg)) => println!("{}", msg),
                Ok(ExecuteResult::DbChanged(name)) => {
                    println!("Using database '{}'", name);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            },
            CommandMode::Remote(port) => match client::send_query(*port, &self.sql) {
                Ok(response) => {
                    let (kind, msg) = client::parse_response(&response);
                    match kind {
                        "OK" => println!("{}", msg),
                        "DB" => println!("Using database '{}'", msg),
                        _ => {
                            eprintln!("Error: {}", msg);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            },
        }
    }
}
