use std::fs;
use std::path::Path;

use crate::error::DbError;
use crate::constants::{self, CATALOG_ID};
use crate::storage::schema;

pub fn create_database(name: &str) -> Result<(), DbError> {
    let path = constants::db_path(name);

    if Path::new(&path).exists() {
        return Err(DbError::DatabaseAlreadyExists(name.into()));
    }

    let catalog_dir = constants::table_dir_path(name, &CATALOG_ID.to_string());
    fs::create_dir_all(&catalog_dir)?;
    fs::write(constants::catalog_data_path(name), "")?;

    schema::create_initial(name)?;

    Ok(())
}
