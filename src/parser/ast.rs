use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum Statement {
    CreateDatabase(CreateDatabase),
    CreateTable(CreateTable),
    DropDatabase(DropDatabase),
    DropTable(DropTable),
    InsertInto(InsertInto),
    Use(UseDatabase),
    Select(Select),
}

#[derive(Debug, PartialEq)]
pub struct InsertInto {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(i64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, PartialEq)]
pub struct Select {
    pub table: String,
    pub where_clause: Option<WhereClause>,
    pub limit: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub struct WhereClause {
    pub column: String,
    pub value: Value,
}

#[derive(Debug, PartialEq)]
pub struct DropDatabase {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct DropTable {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct UseDatabase {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct CreateDatabase {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct CreateTable {
    pub name: String,
    pub columns: Vec<ColumnDef>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub is_primary_key: bool,
    pub is_not_null: bool,
    pub is_unique: bool,
    pub default: Option<DefaultValue>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum DataType {
    Serial,
    Integer,
    Varchar(usize),
    Text,
    Boolean,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum DefaultValue {
    Bool(bool),
    Number(i64),
    String(String),
}
