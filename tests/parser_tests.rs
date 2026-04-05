use db::parser::ast::*;
use db::parser::parse_sql;

#[test]
fn test_parse_create_database() {
    let stmt = parse_sql("CREATE DATABASE mi_app;").unwrap();
    assert_eq!(
        stmt,
        Statement::CreateDatabase(CreateDatabase { name: "mi_app".into() })
    );
}

#[test]
fn test_parse_create_database_no_semicolon() {
    let stmt = parse_sql("CREATE DATABASE test").unwrap();
    assert_eq!(
        stmt,
        Statement::CreateDatabase(CreateDatabase { name: "test".into() })
    );
}

#[test]
fn test_parse_create_table_simple() {
    let stmt = parse_sql("CREATE TABLE usuarios (id INTEGER);").unwrap();
    assert_eq!(
        stmt,
        Statement::CreateTable(CreateTable {
            name: "usuarios".into(),
            columns: vec![
                ColumnDef {
                    name: "id".into(),
                    data_type: DataType::Integer,
                    is_primary_key: false,
                    is_not_null: false,
                    is_unique: false,
                    default: None,
                },
            ],
        })
    );
}

#[test]
fn test_parse_create_table_full() {
    let sql = "CREATE TABLE usuarios (
        id SERIAL PRIMARY KEY,
        email VARCHAR(255) NOT NULL UNIQUE,
        nombre VARCHAR(100) NOT NULL,
        activo BOOLEAN DEFAULT true
    )";
    let stmt = parse_sql(sql).unwrap();
    assert_eq!(
        stmt,
        Statement::CreateTable(CreateTable {
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
                    name: "email".into(),
                    data_type: DataType::Varchar(255),
                    is_primary_key: false,
                    is_not_null: true,
                    is_unique: true,
                    default: None,
                },
                ColumnDef {
                    name: "nombre".into(),
                    data_type: DataType::Varchar(100),
                    is_primary_key: false,
                    is_not_null: true,
                    is_unique: false,
                    default: None,
                },
                ColumnDef {
                    name: "activo".into(),
                    data_type: DataType::Boolean,
                    is_primary_key: false,
                    is_not_null: false,
                    is_unique: false,
                    default: Some(DefaultValue::Bool(true)),
                },
            ],
        })
    );
}

#[test]
fn test_parse_default_number() {
    let stmt = parse_sql("CREATE TABLE t (x INTEGER DEFAULT 0)").unwrap();
    if let Statement::CreateTable(ct) = stmt {
        assert_eq!(ct.columns[0].default, Some(DefaultValue::Number(0)));
    } else {
        panic!("Expected CreateTable");
    }
}

#[test]
fn test_parse_default_string() {
    let stmt = parse_sql("CREATE TABLE t (x TEXT DEFAULT 'hello')").unwrap();
    if let Statement::CreateTable(ct) = stmt {
        assert_eq!(ct.columns[0].default, Some(DefaultValue::String("hello".into())));
    } else {
        panic!("Expected CreateTable");
    }
}

#[test]
fn test_parse_missing_name() {
    let result = parse_sql("CREATE DATABASE;");
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_sql() {
    let result = parse_sql("CREATE");
    assert!(result.is_err());
}

#[test]
fn test_parse_create_table_no_parens() {
    let result = parse_sql("CREATE TABLE t;");
    assert!(result.is_err());
}

#[test]
fn test_parse_drop_table() {
    let stmt = parse_sql("DROP TABLE usuarios;").unwrap();
    assert_eq!(
        stmt,
        Statement::DropTable(DropTable { name: "usuarios".into() })
    );
}

#[test]
fn test_parse_drop_table_no_semicolon() {
    let stmt = parse_sql("DROP TABLE t").unwrap();
    assert_eq!(
        stmt,
        Statement::DropTable(DropTable { name: "t".into() })
    );
}

#[test]
fn test_parse_drop_database() {
    let stmt = parse_sql("DROP DATABASE mi_app;").unwrap();
    assert_eq!(
        stmt,
        Statement::DropDatabase(DropDatabase { name: "mi_app".into() })
    );
}

#[test]
fn test_parse_insert_into() {
    let stmt = parse_sql("INSERT INTO usuarios (email, nombre) VALUES ('john@test.com', 'John');").unwrap();
    assert_eq!(
        stmt,
        Statement::InsertInto(InsertInto {
            table: "usuarios".into(),
            columns: vec!["email".into(), "nombre".into()],
            values: vec![Value::String("john@test.com".into()), Value::String("John".into())],
        })
    );
}

#[test]
fn test_parse_insert_with_null() {
    let stmt = parse_sql("INSERT INTO t (x) VALUES (NULL)").unwrap();
    if let Statement::InsertInto(ins) = stmt {
        assert_eq!(ins.values, vec![Value::Null]);
    } else {
        panic!("Expected InsertInto");
    }
}

#[test]
fn test_parse_insert_with_bool_and_number() {
    let stmt = parse_sql("INSERT INTO t (a, b) VALUES (true, 42)").unwrap();
    if let Statement::InsertInto(ins) = stmt {
        assert_eq!(ins.values, vec![Value::Bool(true), Value::Number(42)]);
    } else {
        panic!("Expected InsertInto");
    }
}

#[test]
fn test_parse_insert_column_value_mismatch() {
    let result = parse_sql("INSERT INTO t (a, b) VALUES (1)");
    assert!(result.is_err());
}
