use std::collections::HashMap;
use std::fs;

use crate::error::DbError;
use crate::parser::ast::{CreateTable, InsertInto};
use crate::storage;
use crate::storage::catalog::{find_by_name, parse_catalog};
use crate::constants;
use crate::storage::schema;

use super::table::Table;

pub struct Database {
    pub name: String,
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn load(name: &str) -> Result<Self, DbError> {
        if !std::path::Path::new(&constants::db_path(name)).exists() {
            return Err(DbError::DatabaseNotFound(name.into()));
        }

        let db_schema = schema::load(name)?;
        let catalog_content = fs::read_to_string(constants::catalog_data_path(name))?;
        let entries = parse_catalog(&catalog_content);

        let mut tables = HashMap::new();
        for entry in &entries {
            if let Some(table_schema) = db_schema.get(&entry.id) {
                let table = Table::load(name, &entry.id, &entry.name, table_schema.clone());
                tables.insert(entry.name.clone(), table);
            }
        }

        Ok(Self {
            name: name.to_string(),
            tables,
        })
    }

    pub fn get_table_mut(&mut self, name: &str) -> Result<&mut Table, DbError> {
        self.tables
            .get_mut(name)
            .ok_or(DbError::TableNotFound(name.into()))
    }

    pub fn insert(&mut self, insert: &InsertInto) -> Result<String, DbError> {
        let table = self.get_table_mut(&insert.table)?;
        table.insert(insert)
    }

    pub fn create_table(&mut self, create: &CreateTable) -> Result<String, DbError> {
        storage::create_table(&self.name, create)?;

        let db_schema = schema::load(&self.name)?;
        let catalog_content = fs::read_to_string(constants::catalog_data_path(&self.name))?;
        let entries = parse_catalog(&catalog_content);

        if let Some(entry) = find_by_name(&entries, &create.name) {
            if let Some(table_schema) = db_schema.get(&entry.id) {
                let table = Table::new(&self.name, &entry.id, &create.name, table_schema.clone());
                self.tables.insert(create.name.clone(), table);
            }
        }

        Ok(format!("Table '{}' created", create.name))
    }

    pub fn drop_table(&mut self, name: &str) -> Result<String, DbError> {
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
