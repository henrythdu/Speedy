mod app;
mod audio;
mod engine;
mod input;
mod repl;
mod storage;
mod terminal;
mod ui;

use crate::app::App;
use rustyline::error::ReadlineError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _terminal_guard = terminal::TerminalGuard::new()?;
    let mut app = App::new();
    let mut repl = repl::ReplInput::new()?;

    println!("Speedy - RSVP Reader");
    println!("Type :h for help, :q to quit");
    println!();

    loop {
        let readline = repl.readline();

        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                repl.add_history_entry(&line)?;

                let event = repl.to_app_event(&line);
                app.handle_event(event);

                if matches!(app.mode, app::mode::AppMode::Quit) {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
