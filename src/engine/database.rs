use std::collections::HashMap;
use std::fs;

use crate::error::DbError;
use crate::parser::ast::{CreateTable, InsertInto};
use crate::storage;
use crate::storage::constants::{CATALOG_ID, DATA_DIR};
use crate::storage::schema;

use super::table::Table;

pub struct Database {
    pub name: String,
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn load(name: &str) -> Result<Self, DbError> {
        let db_path = format!("{}/{}", DATA_DIR, name);
        if !std::path::Path::new(&db_path).exists() {
            return Err(DbError::DatabaseNotFound(name.into()));
        }

        let db_schema = schema::load(name)?;
        let catalog_path = format!("{}/{}/data", db_path, CATALOG_ID);
        let catalog_content = fs::read_to_string(&catalog_path)?;

        let mut tables = HashMap::new();
        for line in catalog_content.lines() {
            let fields: Vec<&str> = line.split('|').collect();
            if fields.len() >= 2 {
                let table_id = fields[0];
                let table_name = fields[1];
                if let Some(table_schema) = db_schema.get(table_id) {
                    let table = Table::load(name, table_id, table_name, table_schema.clone());
                    tables.insert(table_name.to_string(), table);
                }
            }
        }

        Ok(Self {
            name: name.to_string(),
            tables,
        })
    }

    pub fn get_table(&self, name: &str) -> Result<&Table, DbError> {
        self.tables.get(name).ok_or(DbError::TableNotFound(name.into()))
    }

    pub fn get_table_mut(&mut self, name: &str) -> Result<&mut Table, DbError> {
        self.tables.get_mut(name).ok_or(DbError::TableNotFound(name.into()))
    }

    pub fn insert(&mut self, insert: &InsertInto) -> Result<String, DbError> {
        let table = self.get_table_mut(&insert.table)?;
        table.insert(insert)
    }

    pub fn create_table(&mut self, create: &CreateTable) -> Result<String, DbError> {
        storage::create_table(&self.name, create)?;

        // Reload schema to get the new table_id
        let db_schema = schema::load(&self.name)?;
        let catalog_path = format!("{}/{}/{}/data", DATA_DIR, self.name, CATALOG_ID);
        let catalog_content = fs::read_to_string(&catalog_path)?;

        // Find the new table entry in catalog
        for line in catalog_content.lines() {
            let fields: Vec<&str> = line.split('|').collect();
            if fields.len() >= 2 && fields[1] == create.name {
                let table_id = fields[0];
                if let Some(table_schema) = db_schema.get(table_id) {
                    let table = Table::new(&self.name, table_id, &create.name, table_schema.clone());
                    self.tables.insert(create.name.clone(), table);
                }
                break;
            }
        }

        Ok(format!("Table '{}' created", create.name))
    }

    pub fn drop_table(&mut self, name: &str) -> Result<String, DbError> {
        // Flush buffer before dropping
        if let Some(table) = self.tables.get_mut(name) {
            table.flush()?;
        } else {
            return Err(DbError::TableNotFound(name.into()));
        }

        storage::delete_table(&self.name, name)?;
        self.tables.remove(name);

        Ok(format!("Table '{}' dropped", name))
    }

    pub fn flush_all(&mut self) -> Result<(), DbError> {
        for table in self.tables.values_mut() {
            table.flush()?;
        }
        Ok(())
    }
}
