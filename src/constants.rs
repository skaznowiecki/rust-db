// Storage
pub const DATA_DIR: &str = "./data";
pub const CATALOG_ID: u64 = 1000;
pub const FIRST_TABLE_ID: u64 = 1001;

// Engine
pub const BUFFER_SIZE: usize = 200 * 1024; // 200KB

// Server
pub const DEFAULT_PORT: u16 = 5433;
pub const FLUSH_INTERVAL_SECS: u64 = 5;
pub const PID_FILE: &str = "./data/db.pid";
pub const HISTORY_FILE: &str = "./data/history";

// Path helpers

pub fn db_path(name: &str) -> String {
    format!("{}/{}", DATA_DIR, name)
}

pub fn catalog_data_path(db_name: &str) -> String {
    format!("{}/{}/{}/data", DATA_DIR, db_name, CATALOG_ID)
}

pub fn table_dir_path(db_name: &str, table_id: &str) -> String {
    format!("{}/{}/{}", DATA_DIR, db_name, table_id)
}

pub fn table_data_path(db_name: &str, table_id: &str) -> String {
    format!("{}/{}/{}/data", DATA_DIR, db_name, table_id)
}
