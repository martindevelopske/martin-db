use serde_json::Value;

pub enum Statement {
    CreateTable {
        name: String,
        columns: Vec<ColumnDefinition>,
    },
    Insert {
        table_name: String,
        values: Vec<Value>,
    },
    Select {
        table_name: String,
        columns: Vec<String>,
        join: Option<JoinDefinition>,
    },
}

#[derive(Debug)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub is_primary: bool,
    pub is_unique: bool,
}

#[derive(Debug)]
pub struct JoinDefinition {
    pub table_name: String,
    pub left_column: String,
    pub right_column: String,
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .replace('(', " ( ")
        .replace(')', " ) ")
        .replace(',', " , ")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::parser::tokenize;

    #[test]
    pub fn test_tokenize() {
        let input = "SELECT(a,b)";

        let res = tokenize(input);
        println!("{:?}", res);
    }
}
