//! Configuration file loading with XDG-compliant path resolution.
//!
//! Uses the `directories` crate for cross-platform config paths:
//! - Linux: `~/.config/speedy/config.toml`
//! - macOS: `~/Library/Application Support/speedy/config.toml`
//! - Windows: `%APPDATA%\speedy\config.toml`

use std::fs;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use super::Config;

/// Errors that can occur during configuration loading.
#[derive(Debug, Error)]
pub enum ConfigLoadError {
    /// Failed to read the config file.
    #[error("Failed to read config file: {0}")]
    IoError(#[from] io::Error),

    /// Failed to parse the TOML content.
    #[error("Failed to parse config TOML: {0}")]
    ParseError(#[from] toml::de::Error),
}

/// Errors that can occur during configuration saving.
#[derive(Debug, Error)]
pub enum ConfigSaveError {
    /// Failed to serialize the config to TOML.
    #[error("Failed to serialize config to TOML: {0}")]
    SerializeError(#[from] toml::ser::Error),

    /// Failed to write the config file.
    #[error("Failed to write config file: {0}")]
    IoError(#[from] io::Error),
}

/// Returns the XDG-compliant path to the config file.
///
/// Falls back to `./config.toml` in the current directory if
/// the system config directory cannot be determined.
///
/// # Platform-specific paths
/// - Linux: `~/.config/speedy/config.toml`
/// - macOS: `~/Library/Application Support/speedy/config.toml`
/// - Windows: `%APPDATA%\speedy\config.toml`
pub fn config_path() -> PathBuf {
    directories::ProjectDirs::from("com", "speedy", "speedy")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

/// Loads configuration from a TOML file.
///
/// If `path` is `None`, uses the default XDG-compliant path.
/// If the file doesn't exist, returns default configuration.
/// If the file exists but is unreadable or invalid, returns an error.
///
/// # Arguments
/// * `path` - Optional custom path to config file. Uses default if None.
///
/// # Returns
/// * `Ok(Config)` - Loaded configuration (merged with defaults)
/// * `Err(ConfigLoadError)` - File read or parse error
///
/// # Example
/// ```no_run
/// use speedy::config::load;
/// use std::path::PathBuf;
///
/// // Load from default location
/// let config = load(None).expect("Failed to load config");
///
/// // Load from custom path
/// let config = load(Some(PathBuf::from("./my-config.toml"))).expect("Failed to load config");
/// ```
pub fn load(path: Option<PathBuf>) -> Result<Config, ConfigLoadError> {
    let config_path = path.unwrap_or_else(config_path);

    // If config file doesn't exist, return defaults (per design doc)
    if !config_path.exists() {
        tracing::debug!("Config file not found at {:?}, using defaults", config_path);
        return Ok(Config::default());
    }

    tracing::debug!("Loading config from {:?}", config_path);

    let contents = fs::read_to_string(&config_path)?;
    let mut config: Config = toml::from_str(&contents)?;

    // Validate and clamp values
    config.validate();

    tracing::debug!(
        "Loaded config from {:?}: theme={}, wpm={}",
        config_path,
        config.theme,
        config.timing.wpm
    );

    Ok(config)
}

/// Saves configuration to a TOML file.
///
/// If `path` is `None`, uses the default XDG-compliant path.
/// Creates the config directory if it doesn't exist.
///
/// # Arguments
/// * `config` - The configuration to save.
/// * `path` - Optional custom path to config file. Uses default if None.
///
/// # Returns
/// * `Ok(())` - Configuration saved successfully
/// * `Err(ConfigSaveError)` - Serialization or write error
///
/// # Example
/// ```no_run
/// use speedy::config::{load, save};
/// use std::path::PathBuf;
///
/// // Load, modify, and save
/// let mut config = load(None).expect("Failed to load config");
/// config.theme = "dracula".to_string();
/// save(&config, None).expect("Failed to save config");
/// ```
pub fn save(config: &Config, path: Option<PathBuf>) -> Result<(), ConfigSaveError> {
    let config_path = path.unwrap_or_else(config_path);

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            tracing::debug!("Created config directory: {:?}", parent);
        }
    }

    tracing::debug!("Saving config to {:?}", config_path);

    let toml_content = toml::to_string_pretty(config)?;

    fs::write(&config_path, toml_content)?;

    tracing::debug!(
        "Saved config to {:?}: theme={}, default_wpm={}, ghost_words={}",
        config_path,
        config.theme,
        config.default_wpm,
        config.ghost_words
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_missing_file_returns_defaults() {
        let result = load(Some(PathBuf::from("/nonexistent/path/config.toml")));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.theme, "tokyo-night");
        assert_eq!(config.timing.wpm, 300);
    }

    #[test]
    fn test_load_valid_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let toml_content = r#"
theme = "dracula"

[timing]
wpm = 400
period_multiplier = 2.5
"#;
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = load(Some(temp_file.path().to_path_buf()));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.theme, "dracula");
        assert_eq!(config.timing.wpm, 400);
        assert_eq!(config.timing.period_multiplier, 2.5);
        // Defaults preserved for unspecified fields
        assert_eq!(config.timing.comma_multiplier, 1.5);
    }

    #[test]
    fn test_load_partial_config_uses_defaults() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let toml_content = r#"
[timing]
wpm = 500
"#;
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = load(Some(temp_file.path().to_path_buf()));
        assert!(result.is_ok());
        let config = result.unwrap();
        // Default theme preserved
        assert_eq!(config.theme, "tokyo-night");
        assert_eq!(config.timing.wpm, 500);
    }

    #[test]
    fn test_load_invalid_toml_returns_error() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_toml = r#"
theme = "unclosed string
"#;
        temp_file.write_all(invalid_toml.as_bytes()).unwrap();

        let result = load(Some(temp_file.path().to_path_buf()));
        assert!(result.is_err());
    }

    #[test]
    fn test_wpm_is_clamped_on_load() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let toml_content = r#"
[timing]
wpm = 5000
"#;
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let result = load(Some(temp_file.path().to_path_buf()));
        assert!(result.is_ok());
        let config = result.unwrap();
        // WPM should be clamped to MAX_WPM (1000)
        assert_eq!(config.timing.wpm, 1000);
    }

    #[test]
    fn test_config_path_returns_path() {
        let path = config_path();
        // Should return a path ending in config.toml
        assert!(path.ends_with("config.toml"));
    }

    #[test]
    fn test_save_and_load_roundtrip_default_wpm() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Create config with custom default_wpm
        let config = Config {
            default_wpm: 450,
            ..Default::default()
        };

        // Save
        save(&config, Some(path.clone())).expect("Failed to save config");

        // Load and verify
        let loaded = load(Some(path)).expect("Failed to load config");
        assert_eq!(loaded.default_wpm, 450);
    }

    #[test]
    fn test_save_and_load_roundtrip_theme() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Create config with custom theme
        let config = Config {
            theme: "dracula".to_string(),
            ..Default::default()
        };

        // Save
        save(&config, Some(path.clone())).expect("Failed to save config");

        // Load and verify
        let loaded = load(Some(path)).expect("Failed to load config");
        assert_eq!(loaded.theme, "dracula");
    }

    #[test]
    fn test_save_and_load_roundtrip_ghost_words() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Create config with ghost_words enabled
        let config = Config {
            ghost_words: true,
            ..Default::default()
        };

        // Save
        save(&config, Some(path.clone())).expect("Failed to save config");

        // Load and verify
        let loaded = load(Some(path)).expect("Failed to load config");
        assert!(loaded.ghost_words);
    }

    #[test]
    fn test_save_creates_parent_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("config.toml");

        // Parent directory doesn't exist yet
        assert!(!nested_path.parent().unwrap().exists());

        let config = Config::default();
        save(&config, Some(nested_path.clone())).expect("Failed to save config");

        // Parent directory should now exist
        assert!(nested_path.parent().unwrap().exists());
        assert!(nested_path.exists());
    }
}
