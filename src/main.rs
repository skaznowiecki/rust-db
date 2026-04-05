use std::env;

use db::engine::engine::Engine;
use db::provider::Provider;
use db::provider::command::CommandProvider;
use db::provider::repl::ReplProvider;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    match args[0].as_str() {
        "connect" => {
            let mut engine = Engine::new();
            ReplProvider.run(&mut engine);
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
            let mut engine = match db_name {
                Some(db) => Engine::with_db(&db),
                None => Engine::new(),
            };
            CommandProvider { sql }.run(&mut engine);
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
    eprintln!("  db connect                              Interactive REPL");
    eprintln!("  db exec \"<sql>\"                          Execute SQL");
    eprintln!("  db exec --db <database> \"<sql>\"          Execute SQL on database");
}
