use std::io::{self, Write};

use crate::engine::engine::{Engine, ExecuteResult};
use super::client::{self, Connection};
use super::Provider;

pub enum ReplMode {
    Local,
    Remote(u16),
}

pub struct ReplProvider {
    pub mode: ReplMode,
}

impl Provider for ReplProvider {
    fn run(&self, engine: &mut Engine) {
        println!("db v0.1.0");

        match &self.mode {
            ReplMode::Local => {
                println!("Running in local mode.\n");
                self.run_local(engine);
            }
            ReplMode::Remote(port) => {
                println!("Connected to server on port {}.\n", port);
                self.run_remote(*port);
            }
        }
    }
}

impl ReplProvider {
    fn run_local(&self, engine: &mut Engine) {
        println!("Type your SQL statements. Ctrl+C to exit.\n");

        loop {
            let prompt = match engine.current_db() {
                Some(db) => format!("{}> ", db),
                None => "db> ".into(),
            };
            print!("{}", prompt);
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                }
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            match engine.execute(input) {
                Ok(ExecuteResult::Message(msg)) => println!("{}", msg),
                Ok(ExecuteResult::DbChanged(name)) => {
                    println!("Using database '{}'", name);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    fn run_remote(&self, port: u16) {
        let mut conn = match Connection::connect(port) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        };

        println!("Type your SQL statements. Ctrl+C to exit.\n");

        let mut current_db: Option<String> = None;

        loop {
            let prompt = match &current_db {
                Some(db) => format!("{}> ", db),
                None => "db> ".into(),
            };
            print!("{}", prompt);
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                }
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            match conn.send(input) {
                Ok(response) => {
                    let (kind, msg) = client::parse_response(&response);
                    match kind {
                        "OK" => println!("{}", msg),
                        "DB" => {
                            println!("Using database '{}'", msg);
                            current_db = Some(msg.to_string());
                        }
                        _ => eprintln!("Error: {}", msg),
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    }
}
