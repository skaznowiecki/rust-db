use crate::parser::ast::{ColumnDef, DataType, DefaultValue};

pub fn catalog_columns() -> Vec<ColumnDef> {
    vec![
        ColumnDef {
            name: "id".into(),
            data_type: DataType::Serial,
            is_primary_key: true,
            is_not_null: true,
            is_unique: false,
            default: None,
        },
        ColumnDef {
            name: "name".into(),
            data_type: DataType::Text,
            is_primary_key: false,
            is_not_null: true,
            is_unique: false,
            default: None,
        },
        ColumnDef {
            name: "engine".into(),
            data_type: DataType::Text,
            is_primary_key: false,
            is_not_null: false,
            is_unique: false,
            default: Some(DefaultValue::String("default".into())),
        },
    ]
}
