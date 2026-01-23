#[derive(Clone, Debug, PartialEq, Default)]
pub enum AppMode {
    #[default]
    Repl,
    Reading,
    Paused,
    Peek,
    Quit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appmode_enum_exists() {
        let _mode = AppMode::Reading;
        let _mode = AppMode::Paused;
        let _mode = AppMode::Repl;
        let _mode = AppMode::Peek;
        let _mode = AppMode::Quit;
    }
}
