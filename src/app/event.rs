/// Application events
#[derive(Debug, PartialEq, Clone)]
pub enum AppEvent {
    LoadFile(String),
    LoadClipboard,
    Quit,
    Help,
    Warning(String),
    InvalidCommand(String),
    None,
}
