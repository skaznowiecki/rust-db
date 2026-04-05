use std::fs;
use std::path::Path;

use crate::error::DbError;
use crate::constants;

pub fn delete_database(name: &str) -> Result<(), DbError> {
    let path = constants::db_path(name);

    if !Path::new(&path).exists() {
        return Err(DbError::DatabaseNotFound(name.into()));
    }

    fs::remove_dir_all(&path)?;

    Ok(())
}
