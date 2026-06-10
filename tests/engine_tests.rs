use std::fs;
use std::sync::Mutex;

use db::engine::engine::{Engine, ExecuteResult};

static LOCK: Mutex<()> = Mutex::new(());

fn cleanup(db_name: &str) {
    let _ = fs::remove_dir_all(format!("./data/{}", db_name));
}

#[test]
fn test_engine_create_database() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_engine_create_db";
    cleanup(db);

    let mut engine = Engine::new();
    let result = engine.execute("CREATE DATABASE test_engine_create_db").unwrap();
    assert!(matches!(result, ExecuteResult::Message(_)));

    cleanup(db);
}

#[test]
fn test_engine_create_table_without_db() {
    let _guard = LOCK.lock().unwrap();
    let mut engine = Engine::new();
    let result = engine.execute("CREATE TABLE t (id INTEGER)");
    assert!(result.is_err());
}

#[test]
fn test_engine_use_then_create_table() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_engine_use_ct";
    cleanup(db);

    let mut engine = Engine::new();
    engine.execute("CREATE DATABASE test_engine_use_ct").unwrap();
    engine.execute("USE test_engine_use_ct").unwrap();
    assert_eq!(engine.current_db(), Some("test_engine_use_ct"));

    let result = engine.execute("CREATE TABLE t (id INTEGER)").unwrap();
    assert!(matches!(result, ExecuteResult::Message(_)));

    cleanup(db);
}

#[test]
fn test_engine_with_db() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_engine_with_db";
    cleanup(db);

    let mut setup = Engine::new();
    setup.execute("CREATE DATABASE test_engine_with_db").unwrap();

    let mut engine = Engine::with_db("test_engine_with_db");
    let result = engine.execute("CREATE TABLE t (id INTEGER)").unwrap();
    assert!(matches!(result, ExecuteResult::Message(_)));

    cleanup(db);
}

#[test]
fn test_engine_show_databases() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_engine_show_db";
    cleanup(db);

    let mut engine = Engine::new();
    engine.execute("CREATE DATABASE test_engine_show_db").unwrap();

    let result = engine.execute("SHOW DATABASES").unwrap();
    if let ExecuteResult::Message(msg) = result {
        assert!(msg.contains("test_engine_show_db"));
    } else {
        panic!("Expected Message result");
    }

    cleanup(db);
}

#[test]
fn test_engine_drop_database_clears_current() {
    let _guard = LOCK.lock().unwrap();
    let db = "test_engine_drop_db";
    cleanup(db);

    let mut engine = Engine::new();
    engine.execute("CREATE DATABASE test_engine_drop_db").unwrap();
    engine.execute("USE test_engine_drop_db").unwrap();
    assert_eq!(engine.current_db(), Some("test_engine_drop_db"));

    engine.execute("DROP DATABASE test_engine_drop_db").unwrap();
    assert_eq!(engine.current_db(), None);
}
