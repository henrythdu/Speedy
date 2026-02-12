//! CLI argument parsing for Speedy.
//!
//! This module provides command-line argument parsing using clap.

use clap::Parser;
use std::path::PathBuf;

/// Available theme names for the TUI.
const THEME_NAMES: &[&str] = &[
    "tokyo-night",
    "dracula",
    "gruvbox",
    "catppuccin-mocha",
    "nord",
    "light",
];

/// Speedy - A speed reader TUI application.
#[derive(Parser, Debug)]
#[command(name = "speedy")]
#[command(about = "A speed reader TUI application", long_about = None)]
pub struct Args {
    /// Path to a custom configuration file.
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// List available themes and exit.
    #[arg(long)]
    pub list_themes: bool,
}

/// Prints the list of available theme names to stdout.
pub fn list_themes() {
    println!("Available themes:");
    for theme in THEME_NAMES {
        println!("  {}", theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_default() {
        let args = Args::try_parse_from(["speedy"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.config.is_none());
        assert!(!args.list_themes);
    }

    #[test]
    fn test_args_with_config() {
        let args = Args::try_parse_from(["speedy", "--config", "/path/to/config.toml"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.config, Some(PathBuf::from("/path/to/config.toml")));
        assert!(!args.list_themes);
    }

    #[test]
    fn test_args_with_short_config() {
        let args = Args::try_parse_from(["speedy", "-c", "config.toml"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.config, Some(PathBuf::from("config.toml")));
    }

    #[test]
    fn test_args_with_list_themes() {
        let args = Args::try_parse_from(["speedy", "--list-themes"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.list_themes);
    }

    #[test]
    fn test_args_combined() {
        let args = Args::try_parse_from(["speedy", "-c", "my.toml", "--list-themes"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.config, Some(PathBuf::from("my.toml")));
        assert!(args.list_themes);
    }

    #[test]
    fn test_theme_names_count() {
        assert_eq!(THEME_NAMES.len(), 6);
    }
}
