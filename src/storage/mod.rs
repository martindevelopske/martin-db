use crate::{Database, DbError};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub const DB_FILE: &str = "database.json";

pub fn save_to_disk(db: &Database) -> Result<(), DbError> {
    let json = serde_json::to_string_pretty(db)
        .map_err(|err| DbError::IoError(format!("Serialization failed: {}", err)))?;

    let mut file = File::create(DB_FILE)
        .map_err(|err| DbError::IoError(format!("failed to create database file: {}", err)))?;

    file.write_all(json.as_bytes())
        .map_err(|err| DbError::IoError(format!(" Write to database file failed: {}", err)))?;

    Ok(())
}
pub fn load_from_disk() -> Result<Database, DbError> {
    if !Path::new(DB_FILE).exists() {
        return Ok(Database::new());
    }

    let mut file =
        File::open(DB_FILE).map_err(|e| DbError::IoError(format!("Could not open file: {}", e)))?;

    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .map_err(|e| DbError::IoError(format!("Read file failed: {}", e)))?;

    let mut db: Database = serde_json::from_str(&contents)
        .map_err(|e| DbError::IoError(format!("Deserialization failed:{}", e)))?;

    //rebuild indexes sinces we skipped them during Deserialization
    for table in db.tables.values_mut() {
        table.rebuild_indexes();
    }
    Ok(db)
}
