use crate::parser::ast::{ColumnDef, DataType, DefaultValue};

pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub engine: String,
}

pub fn parse_catalog(content: &str) -> Vec<CatalogEntry> {
    content
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('|').collect();
            if fields.len() >= 3 {
                Some(CatalogEntry {
                    id: fields[0].to_string(),
                    name: fields[1].to_string(),
                    engine: fields[2].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn find_by_name<'a>(entries: &'a [CatalogEntry], name: &str) -> Option<&'a CatalogEntry> {
    entries.iter().find(|e| e.name == name)
}

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
