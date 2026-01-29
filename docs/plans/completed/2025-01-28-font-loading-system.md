# Font Loading System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a font loading system that embeds JetBrains Mono Regular OTF font (300KB) into the binary, loads it on startup, and provides font metrics for OVP calculation.

**Architecture:** Use `include_bytes!` macro to embed the font binary. Use `lazy_static` for singleton font initialization. Use `ab_glyph` crate for font parsing and metrics access. Support optional config override for font path.

**Tech Stack:** Rust, ab_glyph (0.2.32), lazy_static (1.5)

---

## Task 1: Add Dependencies to Cargo.toml

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add font loading dependencies**

```toml
[dependencies]
# ... existing dependencies ...
ab_glyph = "0.2.32"
lazy_static = "1.5"
```

**Step 2: Commit**

```bash
git add Cargo.toml
git commit -m "deps: add ab_glyph and lazy_static for font loading"
```

---

## Task 2: Create Font Loading Module with Tests

**Files:**
- Create: `src/engine/font.rs`
- Modify: `src/engine/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_loads_from_embedded_bytes() {
        let font = get_font();
        assert!(font.is_some(), "Font should load from embedded bytes");
    }

    #[test]
    fn test_font_provides_metrics() {
        let font = get_font().expect("Font should be available");
        let scale = PxScale::from(24.0);
        let metrics = font.as_scaled(scale);
        
        assert!(metrics.ascent() > 0.0, "Font should have positive ascent");
        assert!(metrics.descent() < 0.0, "Font should have negative descent");
        assert!(metrics.line_gap() >= 0.0, "Font should have non-negative line gap");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_font_loads_from_embedded_bytes --lib`

Expected: FAIL with "function `get_font` not found"

**Step 3: Write minimal implementation**

```rust
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use lazy_static::lazy_static;

const JETBRAINS_MONO_BYTES: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.otf");

lazy_static! {
    static ref EMBEDDED_FONT: Option<FontRef<'static>> = {
        FontRef::try_from_slice(JETBRAINS_MONO_BYTES).ok()
    };
}

pub fn get_font() -> Option<FontRef<'static>> {
    *EMBEDDED_FONT
}
```

**Step 4: Update module exports**

In `src/engine/mod.rs`, add:
```rust
pub mod font;
```

**Step 5: Run test to verify it passes**

Run: `cargo test test_font_loads_from_embedded_bytes --lib`

Expected: PASS

**Step 6: Commit**

```bash
git add src/engine/font.rs src/engine/mod.rs Cargo.toml Cargo.lock
git commit -m "feat: add embedded font loading with ab_glyph

- Embed JetBrains Mono Regular OTF font (300KB)
- Load font via include_bytes! and lazy_static
- Provide get_font() function for font access
- Add tests for font loading and metrics"
```

---

## Task 3: Download JetBrains Mono Font

**Files:**
- Create: `assets/fonts/JetBrainsMono-Regular.otf`
- Create: `assets/fonts/README.md`

**Step 1: Download the font**

```bash
mkdir -p assets/fonts
cd assets/fonts
curl -L -o JetBrainsMono-Regular.otf \
  "https://github.com/JetBrains/JetBrainsMono/raw/master/fonts/otf/JetBrainsMono-Regular.otf"
cd ../..
```

**Step 2: Create fonts README**

```markdown
# Fonts Directory

This directory contains embedded fonts for Speedy.

## JetBrains Mono

- **File:** `JetBrainsMono-Regular.otf`
- **Source:** https://www.jetbrains.com/lp/mono/
- **License:** Apache 2.0 (see `licenses/JetBrainsMono-LICENSE.txt`)
- **Purpose:** Primary font for RSVP rendering and OVP calculation
- **Embedded Size:** ~300KB

The font is embedded in the binary using Rust's `include_bytes!` macro.
```

**Step 3: Verify font file size**

Run: `ls -lh assets/fonts/JetBrainsMono-Regular.otf`

Expected: ~300KB file exists

**Step 4: Commit**

```bash
git add assets/fonts/
git commit -m "assets: add JetBrains Mono Regular font (300KB)

- Add OTF font file for embedded loading
- Add README with font attribution"
```

---

## Task 4: Add Font License File

**Files:**
- Create: `licenses/JetBrainsMono-LICENSE.txt`
- Modify: `README.md` (add attribution)

**Step 1: Download Apache 2.0 license**

```bash
mkdir -p licenses
curl -L -o licenses/JetBrainsMono-LICENSE.txt \
  "https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/OFL.txt"
```

**Step 2: Update README.md with attribution**

Add section at end of README:
```markdown
## License

### Fonts

This project bundles JetBrains Mono, which is licensed under the Apache 2.0 License.
See `licenses/JetBrainsMono-LICENSE.txt` for the full license text.

Copyright 2020 The JetBrains Mono Project Authors
(https://github.com/JetBrains/JetBrainsMono)
```

**Step 3: Commit**

```bash
git add licenses/ README.md
git commit -m "docs: add JetBrains Mono license and attribution"
```

---

## Task 5: Add Font Metrics Helper Functions

**Files:**
- Modify: `src/engine/font.rs`

**Step 1: Write failing test for character width**

```rust
#[test]
fn test_calculate_character_width() {
    let font = get_font().expect("Font should be available");
    let width = calculate_char_width(&font, 'W', 24.0);
    
    assert!(width > 0.0, "Character width should be positive");
    assert!(width > calculate_char_width(&font, 'i', 24.0), 
            "'W' should be wider than 'i'");
}

#[test]
fn test_calculate_string_width() {
    let font = get_font().expect("Font should be available");
    let width = calculate_string_width(&font, "Hello", 24.0);
    
    assert!(width > 0.0, "String width should be positive");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_calculate_character_width --lib`

Expected: FAIL with "function not found"

**Step 3: Implement font metrics functions**

```rust
pub fn calculate_char_width(font: &FontRef, c: char, font_size: f32) -> f32 {
    let scale = PxScale::from(font_size);
    let scaled_font = font.as_scaled(scale);
    
    font.glyph_id(c)
        .map(|glyph_id| {
            let glyph = font.glyph(glyph_id);
            scaled_font.h_advance(glyph)
        })
        .unwrap_or(font_size * 0.5) // Fallback to half font size
}

pub fn calculate_string_width(font: &FontRef, text: &str, font_size: f32) -> f32 {
    text.chars()
        .map(|c| calculate_char_width(font, c, font_size))
        .sum()
}

pub fn get_font_metrics(font: &FontRef, font_size: f32) -> FontMetrics {
    let scale = PxScale::from(font_size);
    let metrics = font.as_scaled(scale);
    
    FontMetrics {
        ascent: metrics.ascent(),
        descent: metrics.descent(),
        line_gap: metrics.line_gap(),
        height: metrics.height(),
        font_size,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub height: f32,
    pub font_size: f32,
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test font:: --lib`

Expected: All 4 tests PASS

**Step 5: Commit**

```bash
git add src/engine/font.rs
git commit -m "feat: add font metrics calculation functions

- calculate_char_width(): Get width of single character
- calculate_string_width(): Get width of string
- get_font_metrics(): Get full font metrics
- FontMetrics struct for OVP calculations"
```

---

## Task 6: Add Config-Based Font Override

**Files:**
- Modify: `src/engine/font.rs`
- Modify: `src/engine/config.rs` (if exists) or note for future

**Step 1: Write failing test for config override**

```rust
use std::path::Path;

#[test]
fn test_load_font_from_path() {
    // This test uses the embedded font path for simplicity
    let font = load_font_from_path("assets/fonts/JetBrainsMono-Regular.otf");
    assert!(font.is_some(), "Should load font from file path");
}

#[test]
fn test_load_font_from_invalid_path() {
    let font = load_font_from_path("/nonexistent/font.ttf");
    assert!(font.is_none(), "Should return None for invalid path");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_load_font_from_path --lib`

Expected: FAIL with "function not found"

**Step 3: Implement font loading from path**

```rust
use std::fs;
use std::path::Path;

pub fn load_font_from_path<P: AsRef<Path>>(path: P) -> Option<FontRef<'static>> {
    fs::read(path).ok().and_then(|bytes| {
        // Leak the bytes to get 'static lifetime
        let leaked_bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        FontRef::try_from_slice(leaked_bytes).ok()
    })
}

pub struct FontConfig {
    pub custom_font_path: Option<String>,
    pub font_size: f32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            custom_font_path: None,
            font_size: 24.0,
        }
    }
}

pub fn get_font_with_config(config: &FontConfig) -> Option<FontRef<'static>> {
    config.custom_font_path.as_ref()
        .and_then(|path| load_font_from_path(path))
        .or_else(get_font)
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test font:: --lib`

Expected: All 6 tests PASS

**Step 5: Commit**

```bash
git add src/engine/font.rs
git commit -m "feat: add configurable font loading

- load_font_from_path(): Load font from filesystem
- FontConfig struct for font configuration
- get_font_with_config(): Use custom font or fallback to embedded
- Support for user-specified font paths"
```

---

## Task 7: Integrate with Main Application

**Files:**
- Modify: `src/main.rs`
- Modify: `src/engine/capability.rs` (to store font reference)

**Step 1: Add font initialization to main**

```rust
use crate::engine::font::{get_font, get_font_metrics};

fn main() {
    // ... existing capability detection ...
    
    // Initialize font
    match get_font() {
        Some(font) => {
            let metrics = get_font_metrics(&font, 24.0);
            debug!("Font loaded: height={}", metrics.height);
        }
        None => {
            eprintln!("Warning: Failed to load embedded font");
            std::process::exit(1);
        }
    }
    
    // ... rest of main ...
}
```

**Step 2: Run the application to verify**

Run: `cargo run -- --help`

Expected: No errors, help displays

**Step 3: Run all tests**

Run: `cargo test`

Expected: All tests pass (124 + 6 new = 130)

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: integrate font loading into main application

- Initialize font on startup
- Log font metrics for debugging
- Exit gracefully if font fails to load"
```

---

## Task 8: Update Architecture Documentation

**Files:**
- Modify: `docs/ARCHITECTURE.md`

**Step 1: Add font module documentation**

Add section under "Engine Module (src/engine/)" and "Core Types/Structs":

```markdown
### src/engine/font.rs

Font loading and metrics calculation for RSVP rendering.

**Responsibilities:**
- Embed JetBrains Mono Regular OTF font (300KB) via `include_bytes!`
- Load font at startup using `lazy_static` singleton
- Calculate character widths and string widths for OVP positioning
- Provide font metrics (ascent, descent, line_gap, height)
- Support optional custom font path override via config

**Public API:**
- `get_font()` -> `Option<FontRef<'static>>` - Get embedded font singleton
- `load_font_from_path(path)` -> `Option<FontRef<'static>>` - Load from filesystem
- `get_font_with_config(config)` -> `Option<FontRef<'static>>` - Config-based loading
- `calculate_char_width(font, c, font_size)` -> `f32` - Single character width
- `calculate_string_width(font, text, font_size)` -> `f32` - Full string width
- `get_font_metrics(font, font_size)` -> `FontMetrics` - Complete metrics

**Types:**
- `FontConfig` - Configuration for font loading (custom path, size)
- `FontMetrics` - Font metric data (ascent, descent, line_gap, height, font_size)

**Key Dependencies:** `ab_glyph`, `lazy_static`
```

**Step 2: Update Last Updated date**

**Step 3: Commit**

```bash
git add docs/ARCHITECTURE.md
git commit -m "docs: update architecture doc with font module

- Document font loading system
- Add public API reference
- Update Last Updated date"
```

---

## Verification Checklist

Before marking complete:

- [ ] All 6 font module tests pass
- [ ] All existing tests still pass (124 tests)
- [ ] Application runs without errors
- [ ] Font file included in git (assets/fonts/)
- [ ] License file included (licenses/)
- [ ] README updated with attribution
- [ ] Architecture doc updated
- [ ] Dependencies added to Cargo.toml
- [ ] No compiler warnings

---

## Estimated Time

- Task 1: 5 min (add dependencies)
- Task 2: 20 min (create module with TDD)
- Task 3: 10 min (download font)
- Task 4: 10 min (add license)
- Task 5: 20 min (add metrics functions with TDD)
- Task 6: 20 min (add config override with TDD)
- Task 7: 10 min (integrate with main)
- Task 8: 10 min (update docs)

**Total: ~105 minutes (1.75 hours)**
