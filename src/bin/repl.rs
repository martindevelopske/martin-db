use martin_db::{
    parser::{Statement, parse},
    storage::{load_from_disk, save_to_disk},
};
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
        let readline = rl.readline("martin db>>");
        match readline {
            Ok(line) => {
                if line.trim() == "exit" {
                    break;
                }
                if line.trim() == "history" {
                    //read history maybe from a file or something
                    continue;
                }
                if line.trim().is_empty() {
                    continue;
                }

                let trimmed = line.trim();
                match parse(trimmed) {
                    Ok(statement) => {
                        match statement {
                            Statement::CreateTable { name, columns } => {
                                //convert column defs into the engine column
                                let engine_colums = columns
                                    .into_iter()
                                    .map(|c| martin_db::engine::Column {
                                        name: c.name,
                                        data_type: c.data_type,
                                        is_primary: c.is_primary,
                                        is_unique: c.is_unique,
                                    })
                                    .collect();

                                if let Err(e) = db.create_table(name, engine_colums) {
                                    println!("Error: {}", e);
                                } else {
                                    println!("Table created.");
                                    save_to_disk(&db)?;
                                }
                            }
                            _ => println!("Statement Parsed: {:?}", statement),
                        }
                    }
                    Err(e) => {
                        println!("SQL error: {}", e);
                    }
                }

                let _ = rl.add_history_entry(trimmed);
                // println!("Welcome. You entered: {}", line);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }

    Ok(())
}
