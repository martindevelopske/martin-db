use axum::extract::State;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{
    DbError,
    parser::{JoinDefinition, Statement},
};

/// Supported primitive data types for database values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Value {
    Integer(i32),
    Text(String),
    Null,
}

/// Defines the schema of a table column including constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_primary: bool,
    pub is_unique: bool,
}

/// The core data structure for storing records and managing indexes.
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
    /// Creates a new table and initializes empty indexes for primary/unique columns.
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

    /// Inserts a new row into the table after validating constraints.
    /// Returns DbError::UniqueViolation if a PRIMARY or UNIQUE constraint is broken.
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

    /// Reconstructs the in-memory HashSet indexes from the existing rows.
    /// This is called after loading the database from JSON.
    pub fn rebuild_indexes(&mut self) {
        self.indexes.clear();

        //init empty sets for columns that need indexing
        for (i, col) in self.columns.iter().enumerate() {
            if col.is_primary || col.is_unique {
                self.indexes.insert(i, std::collections::HashSet::new());
            }
        }

        //populate sets with existing row data
        for row in &self.rows {
            for (i, value) in row.iter().enumerate() {
                if let Some(index) = self.indexes.get_mut(&i) {
                    index.insert(value.clone());
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub tables: HashMap<String, Table>,
}

/// Possible return values from an executed SQL statement.
pub enum ExecutionResult {
    Message(String),
    Data {
        headers: Vec<String>,
        rows: Vec<Vec<Value>>,
    },
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

    /// Dispatches a parsed Statement to the appropriate internal execution logic.
    pub fn execute(&mut self, statement: Statement) -> Result<ExecutionResult, DbError> {
        match statement {
            Statement::CreateTable { name, columns } => {
                let engine_colums = columns
                    .into_iter()
                    .map(|c| Column {
                        name: c.name,
                        data_type: c.data_type,
                        is_primary: c.is_primary,
                        is_unique: c.is_unique,
                    })
                    .collect();
                self.create_table(name.clone(), engine_colums)?;
                Ok(ExecutionResult::Message(format!(
                    "Table '{}' created",
                    name
                )))
            }
            Statement::Insert { table_name, values } => {
                let table = self
                    .tables
                    .get_mut(&table_name)
                    .ok_or_else(|| DbError::TableNotFound(table_name))?;
                table.insert_row(values)?;
                Ok(ExecutionResult::Message("1 row inserted.".into()))
            }

            Statement::Select {
                table_name,
                columns,
                join,
            } => self.handle_select(table_name, columns, join),
        }
    }

    /// The core execution engine logic for retrieving and combining data.
    ///
    /// ### How it works:
    /// This function branches into two distinct execution paths based on the query:
    ///
    /// #### Path A: Standard Selection (No Join)
    /// 1. **Column Mapping**: Maps requested column names to their numerical indices in the table.
    /// 2. **Projection**: Iterates through `table.rows` and creates a new vector containing only
    ///    the data from the requested indices. This is a linear $O(N)$ operation.
    ///
    /// #### Path B: Inner Join (Nested Loop Join)
    /// 1. **Left/Right Resolution**: Loads both the primary (left) and join (right) tables.
    /// 2. **Index Lookup**: Finds the indices of the columns specified in the `ON` clause.
    /// 3. **Join Algorithm**: Implements a **Nested Loop Join**:
    ///    - Outer Loop: Iterates through every row in the Left Table.
    ///    - Inner Loop: Iterates through every row in the Right Table.
    ///    - Comparison: If `left_row[key] == right_row[key]`, the rows are merged.
    ///    - Complexity: $O(N \times M)$ where $N$ and $M$ are the row counts.
    /// 4. **Header Merging**: Dynamically generates new headers in the format `table.column`
    ///    to prevent naming collisions between joined tables.//
    pub fn handle_select(
        &self,
        table_name: String,
        columns: Vec<String>,
        join: Option<JoinDefinition>,
    ) -> Result<ExecutionResult, DbError> {
        let table = self.get_table(table_name)?;

        //basic select
        if join.is_none() {
            let col_indices: Vec<usize> = if columns.contains(&"*".to_string()) {
                (0..table.columns.len()).collect()
            } else {
                columns
                    .iter()
                    .map(|name| {
                        table
                            .columns
                            .iter()
                            .position(|c| &c.name == name)
                            .ok_or_else(|| DbError::ColumnNotFound(name.clone()))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            };

            let headers = col_indices
                .iter()
                .map(|&i| table.columns[i].name.clone())
                .collect();
            let rows = table
                .rows
                .iter()
                .map(|row| col_indices.iter().map(|&i| row[i].clone()).collect())
                .collect();
            return Ok(ExecutionResult::Data { headers, rows });
        }

        let join_info = join.unwrap();
        let left_table = table;
        let right_table = self.get_table(join_info.table_name)?;

        let left_col_idx = left_table
            .columns
            .iter()
            .position(|c| c.name == join_info.left_column)
            .ok_or_else(|| DbError::ColumnNotFound(join_info.left_column))?;
        let right_col_idx = right_table
            .columns
            .iter()
            .position(|c| c.name == join_info.right_column)
            .ok_or_else(|| DbError::ColumnNotFound(join_info.right_column))?;

        let mut joined_rows = Vec::new();
        let mut headers = Vec::new();

        // Build headers
        for c in &left_table.columns {
            headers.push(format!("{}.{}", left_table.name, c.name));
        }
        for c in &right_table.columns {
            headers.push(format!("{}.{}", right_table.name, c.name));
        }

        // NESTED LOOP JOIN LOGIC
        for l_row in &left_table.rows {
            for r_row in &right_table.rows {
                if l_row[left_col_idx] == r_row[right_col_idx] {
                    let mut combined = l_row.clone();
                    combined.extend(r_row.clone());
                    joined_rows.push(combined);
                }
            }
        }

        Ok(ExecutionResult::Data {
            headers,
            rows: joined_rows,
        })
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
