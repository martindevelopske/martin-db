use rustyline::{DefaultEditor, error::ReadlineError};

fn main() -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;
    println!("Martin Db challenge for pesapal");
    println!("Type 'exit' to quit.");

    loop {
        let readline = rl.readline("martin db>>");
        match readline {
            Ok(line) => {
                //add history to file
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

                println!("Welcome. You entered: {}", line);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => println!("Error: {:?}", err),
        }
    }

    Ok(())
}
