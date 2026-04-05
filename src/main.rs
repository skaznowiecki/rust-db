use std::env;
use std::process::Command;

use db::engine::engine::Engine;
use db::provider::client;
use db::provider::command::{CommandMode, CommandProvider};
use db::provider::repl::{ReplMode, ReplProvider};
use db::provider::server::{self, ServerProvider};
use db::provider::Provider;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let port = client::default_port();

    match args[0].as_str() {
        "start" => {
            if client::try_connect(port) {
                println!("{}", server::server_info(port));
                println!("Server is already running.");
                return;
            }

            // Fork a background process
            let exe = env::current_exe().unwrap();
            let child = Command::new(exe)
                .arg("__server")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();

            match child {
                Ok(c) => {
                    // Wait a moment for it to start
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    if client::try_connect(port) {
                        println!("Server started on 127.0.0.1:{} (pid: {})", port, c.id());
                    } else {
                        eprintln!("Server failed to start");
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to start server: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "__server" => {
            let engine = Engine::new();
            ServerProvider { port }.run_server(engine);
        }
        "stop" => match server::stop_server() {
            Ok(msg) => println!("{}", msg),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        "info" => {
            println!("{}", server::server_info(port));
        }
        "connect" => {
            let mut engine = Engine::new();
            let mode = if client::try_connect(port) {
                ReplMode::Remote(port)
            } else {
                ReplMode::Local
            };
            ReplProvider { mode }.run(&mut engine);
        }
        "exec" => {
            let (db_name, sql) = parse_exec_args(&args[1..]);
            let sql = match sql {
                Some(s) => s,
                None => {
                    eprintln!("Error: exec requires a SQL statement");
                    print_usage();
                    std::process::exit(1);
                }
            };

            let mut engine = Engine::new();

            if client::try_connect(port) {
                if let Some(ref db) = db_name {
                    let use_sql = format!("USE {}", db);
                    if let Err(e) = client::send_query(port, &use_sql) {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
                CommandProvider {
                    sql,
                    mode: CommandMode::Remote(port),
                }
                .run(&mut engine);
            } else {
                if let Some(db) = db_name {
                    engine = Engine::with_db(&db);
                }
                CommandProvider {
                    sql,
                    mode: CommandMode::Local,
                }
                .run(&mut engine);
            }
        }
        other => {
            eprintln!("Unknown command: {}", other);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn parse_exec_args(args: &[String]) -> (Option<String>, Option<String>) {
    let mut db_name = None;
    let mut sql = None;
    let mut i = 0;

    while i < args.len() {
        if args[i] == "--db" {
            if i + 1 < args.len() {
                db_name = Some(args[i + 1].clone());
                i += 2;
            } else {
                eprintln!("Error: --db requires a value");
                std::process::exit(1);
            }
        } else {
            sql = Some(args[i].clone());
            i += 1;
        }
    }

    (db_name, sql)
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  db start                                Start the server (background)");
    eprintln!("  db stop                                 Stop the server");
    eprintln!("  db info                                 Show server status");
    eprintln!("  db connect                              Interactive REPL");
    eprintln!("  db exec \"<sql>\"                          Execute SQL");
    eprintln!("  db exec --db <database> \"<sql>\"          Execute SQL on database");
}
