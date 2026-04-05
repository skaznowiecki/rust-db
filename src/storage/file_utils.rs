use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub fn read_last_line(path: &str) -> Result<Option<String>, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let file_len = file
        .seek(SeekFrom::End(0))
        .map_err(|e| format!("Failed to seek: {}", e))?;

    if file_len == 0 {
        return Ok(None);
    }

    let mut pos = file_len - 1;
    let mut buf = [0u8; 1];

    // Skip trailing newline
    file.seek(SeekFrom::Start(pos))
        .map_err(|e| format!("Failed to seek: {}", e))?;
    file.read_exact(&mut buf)
        .map_err(|e| format!("Failed to read: {}", e))?;
    if buf[0] == b'\n' && pos > 0 {
        pos -= 1;
    }

    // Seek backwards until we find a newline or reach the start
    while pos > 0 {
        file.seek(SeekFrom::Start(pos))
            .map_err(|e| format!("Failed to seek: {}", e))?;
        file.read_exact(&mut buf)
            .map_err(|e| format!("Failed to read: {}", e))?;
        if buf[0] == b'\n' {
            pos += 1;
            break;
        }
        pos -= 1;
    }

    // Read from pos to end
    file.seek(SeekFrom::Start(pos))
        .map_err(|e| format!("Failed to seek: {}", e))?;
    let mut line = String::new();
    file.read_to_string(&mut line)
        .map_err(|e| format!("Failed to read: {}", e))?;

    let line = line.trim_end_matches('\n').to_string();
    if line.is_empty() {
        Ok(None)
    } else {
        Ok(Some(line))
    }
}
