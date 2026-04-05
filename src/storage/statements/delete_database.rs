use std::fs;
use std::path::Path;

use crate::error::DbError;
use crate::storage::constants::DATA_DIR;

pub fn delete_database(name: &str) -> Result<(), DbError> {
    let db_path = format!("{}/{}", DATA_DIR, name);

    if !Path::new(&db_path).exists() {
        return Err(DbError::DatabaseNotFound(name.into()));
    }

    fs::remove_dir_all(&db_path)?;

    Ok(())
}
