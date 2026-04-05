use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use crate::constants::DEFAULT_PORT;

/// Persistent connection to the server
pub struct Connection {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}

impl Connection {
    pub fn connect(port: u16) -> Result<Self, String> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))
            .map_err(|e| format!("Failed to connect to server: {}", e))?;
        let reader = BufReader::new(stream.try_clone().unwrap());
        Ok(Self {
            writer: stream,
            reader,
        })
    }

    pub fn send(&mut self, sql: &str) -> Result<String, String> {
        self.writer
            .write_all(format!("{}\n", sql).as_bytes())
            .map_err(|e| format!("Failed to send query: {}", e))?;

        let mut response = String::new();
        self.reader
            .read_line(&mut response)
            .map_err(|e| format!("Failed to read response: {}", e))?;

        Ok(response.trim_end().to_string())
    }
}

/// One-shot query (opens and closes connection)
pub fn send_query(port: u16, sql: &str) -> Result<String, String> {
    let mut conn = Connection::connect(port)?;
    conn.send(sql)
}

pub fn try_connect(port: u16) -> bool {
    TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok()
}

pub fn default_port() -> u16 {
    DEFAULT_PORT
}

pub fn parse_response(response: &str) -> (&str, &str) {
    if let Some(msg) = response.strip_prefix("OK:") {
        ("OK", msg)
    } else if let Some(msg) = response.strip_prefix("ERR:") {
        ("ERR", msg)
    } else if let Some(name) = response.strip_prefix("DB:") {
        ("DB", name)
    } else {
        ("ERR", response)
    }
}
