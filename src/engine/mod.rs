use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::DbError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Value {
    Integer(i32),
    Text(String),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_primary: bool,
    pub is_unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Value>>,

    // We will use this for fast lookups(indexing) for now. it maps columnIndex -> Hashset of
    // existing values
    #[serde(skip)]
    pub indexes: HashMap<usize, HashSet<Value>>,
}

impl Table {
    pub fn new(name: String, columns: Vec<Column>) -> Self {
        let mut indexes = HashMap::new();
        for (i, col) in columns.iter().enumerate() {
            if col.is_primary || col.is_unique {
                indexes.insert(i, HashSet::new());
            }
        }
        Table {
            name,
            columns,
            rows: Vec::new(),
            indexes,
        }
    }

    pub fn insert_row(&mut self, row: Vec<Value>) -> Result<(), DbError> {
        //validate columns count
        if row.len() != self.columns.len() {
            return Err(DbError::ParseError("Columns count mismatch".into()));
        }

        //check constraints primary or unique
        //If the column has an index (i.e. it is primary or unique)
        // Check if the value already exists
        // If yes → reject the insert
        for (i, value) in row.iter().enumerate() {
            if let Some(index) = self.indexes.get_mut(&i) {
                if index.contains(value) {
                    return Err(DbError::UniqueViolation(self.columns[i].name.clone()));
                }
            }
        }

        //update indexes and push data
        for (i, value) in row.iter().enumerate() {
            if let Some(index) = self.indexes.get_mut(&i) {
                index.insert(value.clone());
            }
        }

        self.rows.push(row);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, name: String, columns: Vec<Column>) -> Result<(), DbError> {
        if self.tables.contains_key(&name) {
            return Err(DbError::TableAlreadyExists(name));
        }

        let table = Table::new(name.clone(), columns);

        self.tables.insert(name, table);
        Ok(())
    }

    pub fn get_table(&self, name: String) -> Result<&Table, DbError> {
        self.tables
            .get(&name)
            .ok_or_else(|| DbError::TableNotFound(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_insert() {
        let mut db = Database::new();
        let cols = vec![
            Column {
                name: "id".into(),
                data_type: "INT".into(),
                is_primary: true,
                is_unique: false,
            },
            Column {
                name: "name".into(),
                data_type: "TEXT".into(),
                is_primary: false,
                is_unique: false,
            },
        ];
        db.create_table("users".into(), cols).unwrap();
        let table = db.tables.get_mut("users").unwrap();

        //first insert -correct
        table
            .insert_row(vec![Value::Integer(1), Value::Text("Martin".into())])
            .unwrap();
        //second_insert - duplicate id
        let badres = table.insert_row(vec![Value::Integer(1), Value::Text("Dup".into())]);
        assert!(badres.is_err());
    }
}
