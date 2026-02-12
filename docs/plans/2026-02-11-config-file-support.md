# Design Doc: Config File Support

**Version:** 1.1
**Date:** 2026-02-11
**Status:** Ready for Implementation
**Conflict Analysis:** Complete (2026-02-11)

---

## 1. Overview

Add user-configurable settings via TOML config file with pre-bundled theme presets.

**Goals:**
- Allow users to customize timing parameters
- Provide pre-built theme presets (no raw color editing)
- Support CLI `--config` flag for custom config paths
- XDG-compliant config location

**Non-Goals:**
- Custom color values via config (use presets only)
- GUI config editor
- Hot-reloading config changes

---

## 2. Config File Schema

**Default location:** `~/.config/speedy/config.toml`

```toml
# Theme preset (see section 3 for options)
theme = "tokyo-night"

# Timing configuration
[timing]
wpm = 300                    # Words per minute (50-1000)
period_multiplier = 3.0      # Pause on . ! ?
comma_multiplier = 1.5       # Pause on ,
question_multiplier = 3.0    # Pause on ?
exclamation_multiplier = 3.0 # Pause on !
newline_multiplier = 4.0     # Pause on newlines
long_word_threshold = 10     # Chars before penalty
long_word_penalty = 1.15     # Extra time multiplier for long words
```

**Partial configs allowed** - missing values use defaults.

---

## 3. Theme Presets

### 3.1 Available Themes

| Theme | Description | Background | Text | Accent |
|-------|-------------|------------|------|--------|
| `tokyo-night` | Default dark (blue-tinted) | #1A1B26 | #A9B1D6 | #F7768E |
| `dracula` | Popular dark theme | #282A36 | #F8F8F2 | #FF79C6 |
| `gruvbox` | Warm, retro feel | #282828 | #EBDBB2 | #FE8019 |
| `catppuccin-mocha` | Pastel, modern | #1E1E2E | #CDD6F4 | #F38BA8 |
| `nord` | Arctic, bluish | #2E3440 | #ECEFF4 | #88C0D0 |
| `light` | Daytime light mode | #FBFBFB | #383A42 | #E45649 |

### 3.2 Theme Color Structure

Each theme defines:

```rust
pub struct ThemeColors {
    pub background: [u8; 4],  // RGBA
    pub surface: [u8; 4],     // RGBA - slightly lighter than bg
    pub text: [u8; 4],        // RGBA - primary text
    pub dimmed: [u8; 4],      // RGBA - secondary/muted text
    pub accent: [u8; 4],      // RGBA - anchor/highlight
}
```

### 3.3 Theme Definitions

```
src/config/themes/
├── mod.rs           # Theme trait and loader
├── tokyo_night.rs   # Default theme
├── dracula.rs
├── gruvbox.rs
├── catppuccin.rs
├── nord.rs
└── light.rs
```

---

## 4. CLI Interface

### 4.1 Arguments

```bash
speedy                    # Use default config (~/.config/speedy/config.toml)
speedy --config ./my.toml # Use custom config
speedy --config /abs/path/to/config.toml
speedy --list-themes      # List available themes
speedy --help
```

### 4.2 Dependency Choice

**Option A: `clap` (Recommended)**
- Declarative, auto-generated help
- Built-in validation
- ~50KB compile cost

**Option B: `std::env::args`**
- Zero dependencies
- Manual parsing
- Sufficient for 2-3 flags

**Decision:** Use `clap` for extensibility and professional UX.

---

## 5. Architecture

### 5.1 Module Structure

```
src/
├── config/
│   ├── mod.rs           # Config loading, merging
│   ├── file.rs          # TOML parsing
│   ├── theme.rs         # Theme preset definitions
│   └── defaults.rs      # Default values (move from engine/config.rs)
├── engine/
│   └── config.rs        # DEPRECATED - move to config/defaults.rs
└── main.rs              # CLI entry point
```

### 5.2 Config Loading Flow

```
main.rs
  │
  ├─► parse CLI args (clap)
  │     └─► config_path: Option<PathBuf>
  │
  ├─► config::load(config_path)
  │     ├─► resolve_path() → XDG or CLI path
  │     ├─► read file (if exists)
  │     ├─► parse TOML
  │     └─► merge with defaults
  │
  └─► App::new(config)
```

### 5.3 Config Struct

```rust
// src/config/mod.rs
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: String,  // Theme name, defaults to "tokyo-night"

    #[serde(default)]
    pub timing: TimingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimingConfig {
    #[serde(default = "default_wpm")]
    pub wpm: u32,

    #[serde(default = "default_period_multiplier")]
    pub period_multiplier: f64,

    // ... other timing fields with defaults
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "tokyo-night".to_string(),
            timing: TimingConfig::default(),
        }
    }
}
```

---

## 6. Implementation Phases

### Phase 1: Foundation
- [ ] Add dependencies: `serde`, `toml`, `directories`, `clap`
- [ ] Create `src/config/mod.rs` with `Config` struct
- [ ] Add `#[derive(Deserialize)]` to config structs
- [ ] Implement `load()` with file parsing

### Phase 2: Themes
- [ ] Create `src/config/theme.rs` with `ThemeColors` struct
- [ ] Implement `tokyo-night` theme (extract current values)
- [ ] Add 5 additional theme presets
- [ ] Create theme resolver: `get_theme(name: &str) -> ThemeColors`

### Phase 3: CLI
- [ ] Add `clap` argument definitions
- [ ] Implement `--config` flag
- [ ] Implement `--list-themes` flag
- [ ] Add `--help` with usage examples

### Phase 4: Integration
- [ ] Wire config into `main.rs`
- [ ] Replace hardcoded values with config references
- [ ] Update `TimingConfig` usage in engine
- [ ] Update `Theme` usage in UI

### Phase 5: Documentation
- [ ] Update README with config section
- [ ] Document theme options
- [ ] Add example config file to repo

---

## 7. File Locations

| Platform | Path |
|----------|------|
| Linux | `~/.config/speedy/config.toml` |
| macOS | `~/Library/Application Support/speedy/config.toml` |
| Windows | `%APPDATA%\speedy\config.toml` |

Use `directories` crate for cross-platform resolution:

```rust
use directories::ProjectDirs;

fn config_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "speedy", "speedy");
    proj_dirs
        .map(|p| p.config_dir().join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}
```

---

## 8. Error Handling

| Scenario | Behavior |
|----------|----------|
| Config file missing | Use defaults, no error |
| Config file unreadable | Warn, use defaults |
| Invalid TOML | Error with line number |
| Unknown theme | Warn, fallback to tokyo-night |
| Invalid timing value | Clamp to valid range |

---

## 9. Testing Strategy

1. **Unit tests:** Config parsing, theme resolution
2. **Integration tests:** File loading with various configs
3. **Manual testing:** Each theme renders correctly

---

## 10. Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
directories = "6.0"
clap = { version = "4.5", features = ["derive"] }
```

---

## 11. Conflict Analysis (Pre-Implementation)

### 11.1 Critical Blockers

| Issue | Location | Solution |
|-------|----------|----------|
| **`RangeInclusive<u32>` cannot deserialize** | `engine/config.rs:48` | Replace with `min_wpm: u32`, `max_wpm: u32` separate fields |
| **`anchor` → `accent` field rename** | 3 UI files, 22+ usages | Find-replace with validation |

### 11.2 Files Requiring Updates

| Impact | File | Changes Needed |
|--------|------|----------------|
| **HIGH** | `src/rendering/kitty/mod.rs` | 10 config constants |
| **MEDIUM** | `src/reading/state.rs` | TimingConfig (8 method calls) |
| **MEDIUM** | `src/rendering/cache.rs` | DEFAULT_FONT_SIZE, DEFAULT_MEMORY_CAP_BYTES |
| **MEDIUM** | `src/ui/terminal.rs` | Theme struct + accent field |
| **MEDIUM** | `src/ui/reader/view.rs` | Theme struct + accent field |
| **LOW** | `src/ui/command_executor.rs` | DEFAULT_WPM |
| **LOW** | `src/main.rs` | DEFAULT_FONT_SIZE, add clap |

### 11.3 Current Config System Analysis

**`src/engine/config.rs`:**
- 16 constants + `TimingConfig` struct
- Private fields (need `pub` for serde)
- `wpm_range: RangeInclusive<u32>` - **cannot impl Deserialize**
- No derives on struct

**`src/ui/theme.rs`:**
- `Theme::midnight()` only, hardcoded
- Uses `ratatui::style::Color` enum (not RGBA)
- Field named `anchor` (rename to `accent`)
- 22+ usages across 3 files

### 11.4 Dependency Status

**Current (Cargo.toml):**
- ❌ `serde` - NOT direct dependency (transitive only)
- ❌ `toml` - NOT present
- ❌ `directories` - NOT present
- ❌ `clap` - NOT present

**All four must be added as new direct dependencies.**

**Cargo.lock Conflicts:**
- 19+ duplicate versions detected
- `getrandom` has 3 versions (0.2.17, 0.3.4, 0.4.1)
- Root cause: Different downstream deps require different API versions
- Recommendation: Run `cargo update` after adding new deps

### 11.5 TimingConfig Refactor Required

```rust
// BEFORE (current) - BLOCKED by serde
pub struct TimingConfig {
    wpm: u32,
    wpm_range: RangeInclusive<u32>,  // ❌ Cannot Deserialize
    // ...
}

// AFTER (proposed) - serde compatible
#[derive(Debug, Clone, Deserialize)]
pub struct TimingConfig {
    #[serde(default = "default_wpm")]
    pub wpm: u32,
    
    // Remove wpm_range, use constants for validation
    // long_word_threshold: usize,
    // long_word_penalty: f64,
    // ... (all other fields pub with serde defaults)
}

impl TimingConfig {
    pub fn validate(&self) {
        self.wpm = self.wpm.clamp(MIN_WPM, MAX_WPM);
    }
}
```

### 11.6 Theme Migration Path

1. Extract current `Theme::midnight()` values → `tokyo-night` preset
2. Convert `ratatui::style::Color` → `[u8; 4]` RGBA
3. Rename `anchor` → `accent` in all files
4. Create `src/config/themes/mod.rs` + 6 preset files
5. Update imports in: `terminal.rs`, `reader/view.rs`, `autocomplete/render.rs`

---

## 12. Future Enhancements

- Custom color override (advanced users)
- Hot-reload config on file change
- Export current config command
- Theme preview command
