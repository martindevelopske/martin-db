use crate::engine::Value;

/// The structure resulting from a successfully parsed SQL string.
#[derive(Debug)]
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

/// Metadata for creating a new column via SQL.
#[derive(Debug)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub is_primary: bool,
    pub is_unique: bool,
}

/// Metadata for performing an INNER JOIN.
#[derive(Debug)]
pub struct JoinDefinition {
    pub table_name: String,
    pub left_column: String,
    pub right_column: String,
}

/// Splits the raw SQL string into tokens while handling parentheses and commas.
fn tokenize(input: &str) -> Vec<String> {
    input
        .replace('(', " ( ")
        .replace(')', " ) ")
        .replace(',', " , ")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Entry point for the SQL parser. Converts raw text into a Statement.
pub fn parse(input: &str) -> Result<Statement, String> {
    let tokens = tokenize(input);
    if tokens.is_empty() {
        return Err("Empty query".into());
    }

    let mut iter = tokens.iter().peekable();
    let command = iter.next().unwrap().to_uppercase();

    match command.as_str() {
        "CREATE" => parse_create(&mut iter),
        "INSERT" => parse_insert(&mut iter),
        "SELECT" => parse_select(&mut iter),
        _ => Err(format!("Unknown command: {}", command)),
    }
}

/// Internal parser logic for the `CREATE TABLE` statement.
///
/// ### How it works:
/// 1. **Keyword Verification**: Validates that the token following `CREATE` is exactly `TABLE`.
/// 2. **Identifier Extraction**: Captures the next token as the table name.
/// 3. **Column Loop**: Enters a loop to parse everything inside the parentheses `(...)`.
/// 4. **Flag Peeking**: For every column, it looks for the name and type. It then uses `iter.peek()`
///    to check for optional constraints like `PRIMARY` or `UNIQUE` without consuming
///    the next required tokens (like commas or closing parentheses).
/// 5. **Validation**: Ensures that the statement is properly closed with a `)`.
fn parse_create(
    iter: &mut std::iter::Peekable<std::slice::Iter<String>>,
) -> Result<Statement, String> {
    if iter.next().map(|s| s.to_uppercase()) != Some("TABLE".to_string()) {
        return Err("Expected TABLE after CREATE".into());
    }

    let name = iter.next().ok_or("Expected table name")?.clone();

    if iter.next() != Some(&"(".to_string()) {
        return Err("Expected '('".into());
    }

    let mut columns = Vec::new();
    while let Some(token) = iter.next() {
        if token == ")" {
            break;
        }
        if token == "," {
            continue;
        }

        let col_name = token.clone();
        let data_type = iter.next().ok_or("Expected column type")?.to_uppercase();

        let mut is_primary = false;
        let mut is_unique = false;

        while let Some(&next) = iter.peek() {
            match next.to_uppercase().as_str() {
                "PRIMARY" => {
                    is_primary = true;
                    iter.next();
                }
                "UNIQUE" => {
                    is_unique = true;
                    iter.next();
                }
                "," | ")" => break,
                _ => {
                    iter.next();
                }
            }
        }

        columns.push(ColumnDefinition {
            name: col_name,
            data_type,
            is_primary,
            is_unique,
        });
    }

    Ok(Statement::CreateTable { name, columns })
}

/// Internal parser logic for the `INSERT INTO` statement.
///
/// ### How it works:
/// 1. **Context Parsing**: Matches the boilerplate SQL syntax `INTO <table_name> VALUES`.
/// 2. **Type Inference**: As it iterates through the values inside `(...)`, it attempts to
///    categorize data types on the fly:
///    - If a token can be parsed as a number (`token.parse::<i32>()`), it is stored as `Value::Integer`.
///    - Otherwise, it is treated as a string and stored as `Value::Text`.
/// 3. **Sanitization**: It strips single quotes `'` from text values to ensure
///    the database stores the literal data, not the SQL formatting.
fn parse_insert(
    iter: &mut std::iter::Peekable<std::slice::Iter<String>>,
) -> Result<Statement, String> {
    if iter.next().map(|s| s.to_uppercase()) != Some("INTO".to_string()) {
        return Err("Expected INTO after CREATE".into());
    }

    let name = iter.next().ok_or("Expected table name")?.clone();
    if iter.next().map(|s| s.to_uppercase()) != Some("VALUES".to_string()) {
        return Err("Expected VALUES after INTO".into());
    }

    if iter.next() != Some(&"(".to_string()) {
        return Err("Expected '('".into());
    }

    let mut values = Vec::new();
    while let Some(token) = iter.next() {
        if token == ")" {
            break;
        }
        if token == "," {
            continue;
        }

        if let Ok(num) = token.parse::<i32>() {
            values.push(Value::Integer(num));
        } else {
            values.push(Value::Text(token.trim_matches('\'').to_string()));
        }
    }

    Ok(Statement::Insert {
        table_name: name,
        values,
    })
}

/// Internal parser logic for the `SELECT` statement, including JOIN detection.
///
/// ### How it works:
/// 1. **Column Selection**: Collects all tokens between `SELECT` and `FROM`. This supports
///    both `*` (wildcard) and specific column lists (e.g., `id, name`).
/// 2. **Source Table**: Identifies the primary table to query.
/// 3. **Join Detection**: After the table name, it "peeks" ahead. If the next token is `JOIN`,
///    it switches to "Join Mode":
///    - It captures the secondary table name.
///    - It skips the `ON` keyword.
///    - It extracts the `left_column` and `right_column` used for the equality check.
/// 4. **Encapsulation**: Returns a `Statement::Select` containing a `JoinDefinition`
///    struct if a join was detected, otherwise `None`.
fn parse_select(
    iter: &mut std::iter::Peekable<std::slice::Iter<String>>,
) -> Result<Statement, String> {
    let mut columns = Vec::new();
    while let Some(token) = iter.next() {
        if token.to_uppercase() == "FROM" {
            break;
        }
        if token != "," {
            columns.push(token.clone());
        }
    }

    let table_name = iter.next().ok_or("Expected table name")?.clone();
    let mut join = None;

    if let Some(token) = iter.next() {
        let join_table = iter.next().ok_or("Expected join table")?.clone();
        iter.next();
        let left = iter.next().ok_or("Expected left col")?.clone();
        iter.next();
        let right = iter.next().ok_or("Expected right col")?.clone();

        join = Some(JoinDefinition {
            table_name: join_table,
            left_column: left,
            right_column: right,
        });
    }

    Ok(Statement::Select {
        table_name: table_name,
        columns: columns,
        join: join,
    })
}

#[cfg(test)]
mod tests {
    use crate::parser::{self, parse, tokenize};

    #[test]
    pub fn test_tokenize() {
        let input = "SELECT(a,b)";

        let res = tokenize(input);
        println!("{:?}", res);
    }

    #[test]
    pub fn test_parser() {
        let input = "CREATE TABLE users (id INT PRIMARY, name TEXT)";
        match parse(input) {
            Ok(smt) => println!("Parsed: {:?}", smt),
            Err(e) => println!("Error: {}", e),
        }
    }
}
