use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use std::io;
use std::sync::Once;

static PANIC_HOOK_SET: Once = Once::new();

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        io::stdout().execute(terminal::EnterAlternateScreen)?;

        set_panic_hook();

        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = io::stdout().execute(terminal::LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn set_panic_hook() {
    PANIC_HOOK_SET.call_once(|| {
        std::panic::set_hook(Box::new(|panic_info| {
            let _ = disable_raw_mode();
            let _ = io::stdout().execute(terminal::LeaveAlternateScreen);
            eprintln!("Panic: {}", panic_info);
            std::process::exit(1);
        }));
    });
}
