mod app;
mod audio;
mod engine;
mod repl;
mod storage;
mod terminal;
mod ui;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::Clear;
use crossterm::{cursor::MoveTo, queue, style::Print, terminal::ClearType};
use std::io::{stdout, Write};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _terminal_guard = terminal::TerminalGuard::new()?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) => match key_event.code {
                    KeyCode::Char('c') if key_event.modifiers == event::KeyModifiers::CONTROL => {
                        break;
                    }
                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') => print_help(&mut stdout())?,
                    _ => {}
                },
                _ => {}
            }
        }
    }

    Ok(())
}

fn print_help(stdout: &mut impl Write) -> std::io::Result<()> {
    queue!(
        stdout,
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print("Speedy - RSVP Reader\n"),
        Print("Commands:\n"),
        Print("  @filename - Load a file\n"),
        Print("  :q - Quit\n"),
        Print("  :h - Help\n")
    )?;
    stdout.flush()
}
