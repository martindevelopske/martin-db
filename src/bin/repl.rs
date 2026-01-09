use martin_db::parser::{Statement, parse};
use rustyline::{DefaultEditor, error::ReadlineError};

fn main() -> anyhow::Result<()> {
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
                        println!("Success! parser detected: {:?}", statement);
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
