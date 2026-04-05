use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use crate::engine::engine::{Engine, ExecuteResult};
use super::Provider;

const PID_FILE: &str = "./data/db.pid";

pub struct ServerProvider {
    pub port: u16,
}

impl Provider for ServerProvider {
    fn run(&self, engine: &mut Engine) {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        });

        // Save PID
        let pid = std::process::id();
        let _ = fs::create_dir_all("./data");
        fs::write(PID_FILE, pid.to_string()).unwrap_or_else(|e| {
            eprintln!("Warning: could not write PID file: {}", e);
        });

        println!("Server listening on {} (pid: {})", addr, pid);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    Self::handle_connection(engine, stream);
                }
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                }
            }
        }
    }
}

pub fn read_pid() -> Option<u32> {
    fs::read_to_string(PID_FILE)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

pub fn is_running() -> bool {
    if let Some(pid) = read_pid() {
        // Check if process is alive
        unsafe { libc_kill(pid) }
    } else {
        false
    }
}

pub fn stop_server() -> Result<String, String> {
    let pid = read_pid().ok_or("Server is not running (no PID file found)")?;

    if !unsafe { libc_kill(pid) } {
        let _ = fs::remove_file(PID_FILE);
        return Err("Server is not running (stale PID file removed)".into());
    }

    // Send SIGTERM
    let status = std::process::Command::new("kill")
        .arg(pid.to_string())
        .status()
        .map_err(|e| format!("Failed to kill process: {}", e))?;

    if status.success() {
        let _ = fs::remove_file(PID_FILE);
        Ok(format!("Server stopped (pid: {})", pid))
    } else {
        Err(format!("Failed to stop server (pid: {})", pid))
    }
}

pub fn server_info(port: u16) -> String {
    if let Some(pid) = read_pid() {
        if unsafe { libc_kill(pid) } {
            format!("Server is running on 127.0.0.1:{} (pid: {})", port, pid)
        } else {
            let _ = fs::remove_file(PID_FILE);
            "Server is not running (stale PID file cleaned up)".into()
        }
    } else {
        "Server is not running".into()
    }
}

/// Check if a process is alive using kill(pid, 0)
unsafe fn libc_kill(pid: u32) -> bool {
    // kill(pid, 0) returns 0 if process exists
    let ret = unsafe { libc::kill(pid as i32, 0) };
    ret == 0
}

impl ServerProvider {
    fn handle_connection(engine: &mut Engine, stream: std::net::TcpStream) {
        let peer = stream.peer_addr().ok();
        let reader = BufReader::new(&stream);
        let mut writer = stream.try_clone().unwrap();

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };

            let sql = line.trim();
            if sql.is_empty() {
                continue;
            }

            let response = match engine.execute(sql) {
                Ok(ExecuteResult::Message(msg)) => format!("OK:{}\n", msg),
                Ok(ExecuteResult::DbChanged(name)) => format!("DB:{}\n", name),
                Err(e) => format!("ERR:{}\n", e),
            };

            if let Some(ref addr) = peer {
                println!("[{}] {} → {}", addr, sql, response.trim());
            }

            if writer.write_all(response.as_bytes()).is_err() {
                break;
            }
        }
    }
}
