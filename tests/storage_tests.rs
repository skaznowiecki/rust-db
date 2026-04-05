use std::fs;
use std::path::Path;
use std::sync::Mutex;

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
    assert!(result.unwrap_err().contains("already exists"));

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
    assert!(result.unwrap_err().contains("already exists"));

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
    assert!(result.unwrap_err().contains("does not exist"));
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
    assert!(result.unwrap_err().contains("does not exist"));

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
    assert!(result.unwrap_err().contains("does not exist"));
}
