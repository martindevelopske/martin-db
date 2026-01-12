use martin_db::{
    engine::ExecutionResult,
    parser::parse,
    storage::{load_from_disk, save_to_disk},
};
use prettytable::{Cell, Row, Table};
use rustyline::{DefaultEditor, error::ReadlineError};

fn main() -> anyhow::Result<()> {
    let mut db = load_from_disk().unwrap_or_else(|_| {
        println!("Initializing a new Database.");
        martin_db::Database::new()
    });

    let mut rl = DefaultEditor::new()?;
    println!("Martin Db challenge for pesapal");
    println!("Type 'exit' to quit.");

    loop {
        let readline = rl.readline("sql> ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed == "exit" {
                    break;
                }

                match parse(trimmed) {
                    Ok(stmt) => {
                        // Check if it's a mutating query to save later
                        let is_mutation = matches!(
                            stmt,
                            martin_db::parser::Statement::CreateTable { .. }
                                | martin_db::parser::Statement::Insert { .. }
                        );

                        match db.execute(stmt) {
                            Ok(result) => {
                                match result {
                                    ExecutionResult::Message(msg) => println!("{}", msg),
                                    ExecutionResult::Data { headers, rows } => {
                                        let mut table = Table::new();
                                        table.add_row(Row::new(
                                            headers.into_iter().map(|s| Cell::new(&s)).collect(),
                                        ));
                                        for r in rows {
                                            table.add_row(Row::new(
                                                r.into_iter()
                                                    .map(|v| Cell::new(&format!("{:?}", v)))
                                                    .collect(),
                                            ));
                                        }
                                        table.printstd();
                                    }
                                }
                                if is_mutation {
                                    save_to_disk(&db)?;
                                }
                            }
                            Err(e) => println!("Execution Error: {}", e),
                        }
                    }
                    Err(e) => println!("Syntax Error: {}", e),
                }
                let _ = rl.add_history_entry(trimmed);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            _ => (),
        }
    }
    Ok(())
}
