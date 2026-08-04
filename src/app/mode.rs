#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum AppMode {
    #[default]
    Command,
    Reading,
    Paused,
    Popup,
    /// Static reference overlay (:help / :h). Blocks all keys except
    /// dismiss — a mode because reading must freeze while it's open.
    Help,
    Quit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appmode_enum_exists() {
        let _mode = AppMode::Reading;
        let _mode = AppMode::Paused;
        let _mode = AppMode::Command;
        let _mode = AppMode::Popup;
        let _mode = AppMode::Help;
        let _mode = AppMode::Quit;
    }
}
