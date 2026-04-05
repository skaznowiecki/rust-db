use std::fs;
use std::path::Path;
use std::sync::Mutex;

use db::error::DbError;
use db::parser::ast::*;
use db::storage;

static LOCK: Mutex<()> = Mutex::new(());

fn cleanup(db_name: &str) {
    let _ = fs::remove_dir_all(format!("./data/{}", db_name));
}

#[test]
fn test_create_database_creates_catalog() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_create_db_catalog";
    cleanup(db);

    storage::create_database(db).unwrap();

    assert!(Path::new(&format!("./data/{}/1000", db)).exists());
    assert!(Path::new(&format!("./data/{}/1000/data", db)).exists());
    // Schema is inside the DB directory
    let content = fs::read_to_string(format!("./data/{}/schema.json", db)).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(schema["1000"]["columns"].is_array());

    cleanup(db);
}

#[test]
fn test_create_database_already_exists() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_create_db_dup";
    cleanup(db);

    storage::create_database(db).unwrap();
    let result = storage::create_database(db);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DbError::DatabaseAlreadyExists(_)));

    cleanup(db);
}

#[test]
fn test_create_table() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_create_table";
    cleanup(db);
    storage::create_database(db).unwrap();

    let table = CreateTable {
        name: "usuarios".into(),
        columns: vec![
            ColumnDef {
                name: "id".into(),
                data_type: DataType::Serial,
                is_primary_key: true,
                is_not_null: false,
                is_unique: false,
                default: None,
            },
            ColumnDef {
                name: "nombre".into(),
                data_type: DataType::Text,
                is_primary_key: false,
                is_not_null: true,
                is_unique: false,
                default: None,
            },
        ],
    };

    storage::create_table(db, &table).unwrap();

    assert!(Path::new(&format!("./data/{}/1001", db)).exists());
    assert!(Path::new(&format!("./data/{}/1001/data", db)).exists());

    let catalog = fs::read_to_string(format!("./data/{}/1000/data", db)).unwrap();
    assert!(catalog.contains("1001|usuarios|default"));

    let content = fs::read_to_string(format!("./data/{}/schema.json", db)).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(schema["1001"]["columns"].is_array());

    cleanup(db);
}

#[test]
fn test_create_table_duplicate() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_create_table_dup";
    cleanup(db);
    storage::create_database(db).unwrap();

    let table = CreateTable {
        name: "t".into(),
        columns: vec![ColumnDef {
            name: "id".into(),
            data_type: DataType::Integer,
            is_primary_key: false,
            is_not_null: false,
            is_unique: false,
            default: None,
        }],
    };

    storage::create_table(db, &table).unwrap();
    let result = storage::create_table(db, &table);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DbError::TableAlreadyExists(_)));

    cleanup(db);
}

#[test]
fn test_create_table_db_not_found() {
    let _guard = LOCK.lock().unwrap();
    let table = CreateTable {
        name: "t".into(),
        columns: vec![ColumnDef {
            name: "id".into(),
            data_type: DataType::Integer,
            is_primary_key: false,
            is_not_null: false,
            is_unique: false,
            default: None,
        }],
    };

    let result = storage::create_table("nonexistent_db_xyz", &table);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DbError::DatabaseNotFound(_)));
}

#[test]
fn test_delete_table() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_delete_table";
    cleanup(db);
    storage::create_database(db).unwrap();

    let table = CreateTable {
        name: "usuarios".into(),
        columns: vec![ColumnDef {
            name: "id".into(),
            data_type: DataType::Integer,
            is_primary_key: false,
            is_not_null: false,
            is_unique: false,
            default: None,
        }],
    };

    storage::create_table(db, &table).unwrap();
    assert!(Path::new(&format!("./data/{}/1001", db)).exists());

    storage::delete_table(db, "usuarios").unwrap();

    // Directory removed
    assert!(!Path::new(&format!("./data/{}/1001", db)).exists());

    // Catalog updated
    let catalog = fs::read_to_string(format!("./data/{}/1000/data", db)).unwrap();
    assert!(!catalog.contains("usuarios"));

    // Schema updated
    let content = fs::read_to_string(format!("./data/{}/schema.json", db)).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(schema.get("1001").is_none());

    cleanup(db);
}

#[test]
fn test_delete_table_not_found() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_delete_table_nf";
    cleanup(db);
    storage::create_database(db).unwrap();

    let result = storage::delete_table(db, "nonexistent");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DbError::TableNotFound(_)));

    cleanup(db);
}

#[test]
fn test_delete_database() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_delete_database";
    cleanup(db);
    storage::create_database(db).unwrap();
    assert!(Path::new(&format!("./data/{}", db)).exists());

    storage::delete_database(db).unwrap();
    assert!(!Path::new(&format!("./data/{}", db)).exists());
}

#[test]
fn test_delete_database_not_found() {
    let _guard = LOCK.lock().unwrap();
    let result = storage::delete_database("nonexistent_db_xyz");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DbError::DatabaseNotFound(_)));
}

// --- INSERT INTO tests ---

fn setup_db_with_table(db_name: &str) -> db::engine::database::Database {
    cleanup(db_name);
    storage::create_database(db_name).unwrap();
    let table = CreateTable {
        name: "users".into(),
        columns: vec![
            ColumnDef { name: "id".into(), data_type: DataType::Serial, is_primary_key: true, is_not_null: true, is_unique: false, default: None },
            ColumnDef { name: "email".into(), data_type: DataType::Varchar(255), is_primary_key: false, is_not_null: true, is_unique: true, default: None },
            ColumnDef { name: "name".into(), data_type: DataType::Text, is_primary_key: false, is_not_null: false, is_unique: false, default: None },
            ColumnDef { name: "active".into(), data_type: DataType::Boolean, is_primary_key: false, is_not_null: false, is_unique: false, default: Some(DefaultValue::Bool(true)) },
        ],
    };
    storage::create_table(db_name, &table).unwrap();
    db::engine::database::Database::load(db_name).unwrap()
}

#[test]
fn test_insert_valid_row() {
    let _guard = LOCK.lock().unwrap();
    let db_name = "test_insert_valid";
    let mut database = setup_db_with_table(db_name);

    let insert = InsertInto {
        table: "users".into(),
        columns: vec!["email".into(), "name".into()],
        values: vec![Value::String("john@test.com".into()), Value::String("John".into())],
    };
    database.insert(&insert).unwrap();
    database.flush_all().unwrap();

    let data = fs::read_to_string(format!("./data/{}/1001/data", db_name)).unwrap();
    assert_eq!(data.trim(), "1|john@test.com|John|true");
    cleanup(db_name);
}

#[test]
fn test_insert_serial_autoincrement() {
    let _guard = LOCK.lock().unwrap();
    let db_name = "test_insert_serial";
    let mut database = setup_db_with_table(db_name);

    let insert1 = InsertInto { table: "users".into(), columns: vec!["email".into()], values: vec![Value::String("a@test.com".into())] };
    let insert2 = InsertInto { table: "users".into(), columns: vec!["email".into()], values: vec![Value::String("b@test.com".into())] };

    database.insert(&insert1).unwrap();
    database.insert(&insert2).unwrap();
    database.flush_all().unwrap();

    let data = fs::read_to_string(format!("./data/{}/1001/data", db_name)).unwrap();
    let lines: Vec<&str> = data.trim().lines().collect();
    assert!(lines[0].starts_with("1|"));
    assert!(lines[1].starts_with("2|"));
    cleanup(db_name);
}

#[test]
fn test_insert_column_not_found() {
    let _guard = LOCK.lock().unwrap();
    let db_name = "test_insert_col_nf";
    let mut database = setup_db_with_table(db_name);

    let insert = InsertInto { table: "users".into(), columns: vec!["nonexistent".into()], values: vec![Value::String("x".into())] };
    let result = database.insert(&insert);
    assert!(matches!(result.unwrap_err(), DbError::ColumnNotFound { .. }));
    cleanup(db_name);
}

#[test]
fn test_insert_type_mismatch() {
    let _guard = LOCK.lock().unwrap();
    let db_name = "test_insert_type";
    let mut database = setup_db_with_table(db_name);

    let insert = InsertInto {
        table: "users".into(),
        columns: vec!["email".into(), "active".into()],
        values: vec![Value::String("a@test.com".into()), Value::Number(42)],
    };
    let result = database.insert(&insert);
    assert!(matches!(result.unwrap_err(), DbError::TypeMismatch { .. }));
    cleanup(db_name);
}

#[test]
fn test_insert_not_null_missing() {
    let _guard = LOCK.lock().unwrap();
    let db_name = "test_insert_nn";
    let mut database = setup_db_with_table(db_name);

    let insert = InsertInto { table: "users".into(), columns: vec!["name".into()], values: vec![Value::String("John".into())] };
    let result = database.insert(&insert);
    assert!(matches!(result.unwrap_err(), DbError::NotNullViolation(_)));
    cleanup(db_name);
}

#[test]
fn test_insert_default_value() {
    let _guard = LOCK.lock().unwrap();
    let db_name = "test_insert_default";
    let mut database = setup_db_with_table(db_name);

    let insert = InsertInto { table: "users".into(), columns: vec!["email".into()], values: vec![Value::String("a@test.com".into())] };
    database.insert(&insert).unwrap();
    database.flush_all().unwrap();

    let data = fs::read_to_string(format!("./data/{}/1001/data", db_name)).unwrap();
    assert!(data.contains("|true"));
    cleanup(db_name);
}

#[test]
fn test_insert_null_value() {
    let _guard = LOCK.lock().unwrap();
    let db_name = "test_insert_null";
    let mut database = setup_db_with_table(db_name);

    let insert = InsertInto {
        table: "users".into(),
        columns: vec!["email".into(), "name".into()],
        values: vec![Value::String("a@test.com".into()), Value::Null],
    };
    database.insert(&insert).unwrap();
    database.flush_all().unwrap();

    let data = fs::read_to_string(format!("./data/{}/1001/data", db_name)).unwrap();
    assert_eq!(data.trim(), "1|a@test.com||true");
    cleanup(db_name);
}
