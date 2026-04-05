use std::io::{self, Write};

use crate::engine::engine::{Engine, ExecuteResult};
use super::Provider;

pub struct ReplProvider;

impl Provider for ReplProvider {
    fn run(&self, engine: &mut Engine) {
        println!("db v0.1.0");
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
}
