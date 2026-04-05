use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::error::DbError;

pub fn read_last_line(path: &str) -> Result<Option<String>, DbError> {
    let mut file = File::open(path)?;

    let file_len = file.seek(SeekFrom::End(0))?;

    if file_len == 0 {
        return Ok(None);
    }

    let mut pos = file_len - 1;
    let mut buf = [0u8; 1];

    file.seek(SeekFrom::Start(pos))?;
    file.read_exact(&mut buf)?;
    if buf[0] == b'\n' && pos > 0 {
        pos -= 1;
    }

    while pos > 0 {
        file.seek(SeekFrom::Start(pos))?;
        file.read_exact(&mut buf)?;
        if buf[0] == b'\n' {
            pos += 1;
            break;
        }
        pos -= 1;
    }

    file.seek(SeekFrom::Start(pos))?;
    let mut line = String::new();
    file.read_to_string(&mut line)?;

    let line = line.trim_end_matches('\n').to_string();
    if line.is_empty() {
        Ok(None)
    } else {
        Ok(Some(line))
    }
}
