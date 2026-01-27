mod app;
mod audio;
mod engine;
mod input;
mod repl;
mod storage;
mod ui;

use crate::app::{mode::AppMode, App};
use crate::ui::TuiManager;
use rustyline::error::ReadlineError;

fn run_tui_mode(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    use crate::ui::TuiManager;

    let mut tui = TuiManager::new()?;

    let final_mode = tui.run_event_loop(app)?;

    app.set_mode(final_mode);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

                if matches!(app.mode, AppMode::Quit) {
                    break;
                }

                if matches!(app.mode, AppMode::Reading) {
                    if let Err(e) = run_tui_mode(&mut app) {
                        eprintln!("TUI Error: {}", e);
                        break;
                    }
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
