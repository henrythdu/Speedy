# Speedy Architecture Document

**Last Updated:** 2026-02-10 (Removed unused SentenceProgressBar module, updated PRD, 0 clippy warnings)
**Purpose:** Document actual codebase structure, methods, structs, and architecture to prevent duplication and confusion.

## ⚠️ Important Notes

1. **This document only describes WHAT EXISTS** - not planned or proposed code
2. **Update when:** Adding new modules, structs, public methods, or changing architecture
3. **Don't update for:** Test-only changes, refactors that don't change public API
4. **Keep concise:** Brief descriptions only, not full documentation
5. **Reference:** Use `file_path:line_number` format for code references

---

## 1. Project Structure

```
src/
├── app/                 # Application layer (state management, UI coordination)
│   ├── app_impl.rs     # Main App struct and business logic
│   ├── mode.rs         # AppMode enum (Command, Reading, Paused, Quit)
│   └── mod.rs          # App module exports
├── engine/             # Shared logic (config only)
│   ├── config.rs       # ReadingConfig timing configuration
│   └── mod.rs          # Engine module (re-exports from reading/ and rendering/)
├── reading/            # Core RSVP reading logic domain
│   ├── token.rs        # Token struct
│   ├── timing.rs       # Tokenization, WPM calculations, sentence boundaries
│   ├── state.rs        # ReadingState with navigation and timing
│   ├── ovp.rs          # OVP anchor position calculation
│   └── mod.rs          # Reading module exports
├── rendering/          # Rendering backends domain
│   ├── cache.rs        # Word-Level LRU cache for rendered buffers
│   ├── renderer.rs     # RsvpRenderer trait and RendererError
│   ├── viewport.rs     # Viewport coordinates and terminal dimensions
│   ├── font.rs         # Font loading and metrics
│   ├── kitty/          # Kitty Graphics Protocol modules
│   │   ├── mod.rs      # KittyGraphicsRenderer implementation
│   │   ├── protocol.rs # KGP transmission and encoding
│   │   ├── rasterizer.rs # Word-to-image rendering
│   │   └── positioning.rs # OVP anchoring calculations
│   └── mod.rs          # Rendering module exports
├── ui/                 # TUI rendering layer
│   ├── reader/         # Reader feature module
│   │   ├── view.rs     # Render functions (OVP word, progress)
│   │   └── mod.rs      # Reader module exports
│   ├── command.rs      # Command parsing and token utilities
│   ├── command_executor.rs # Command execution logic
│   ├── terminal.rs     # TuiManager with event loop and frame rendering
│   ├── theme.rs        # Theme configuration (Midnight colors)
│   └── mod.rs          # UI module exports
├── input/              # File input processing
│   ├── pdf.rs          # PDF parsing
│   ├── epub.rs         # EPUB parsing
│   ├── clipboard.rs    # Clipboard content extraction
│   └── mod.rs          # Input module exports
├── audio/              # Audio feedback (metronome, etc.)
│   └── mod.rs          # Audio module exports
├── storage/            # Persistence (settings, history)
│   └── mod.rs          # Storage module exports
└── main.rs             # Entry point with TUI launch
```

---

## 2. Core Structs

### `App` (`src/app/app_impl.rs:18`)
Main application state container.
```rust
pub struct App {
    mode: AppMode,                     // Current mode (Command/Reading/Paused/Quit)
    reading_state: Option<ReadingState>, // Current reading session
}
```

**Purpose:** Coordinates between TUI and engine layers. Manages mode transitions.

**Key Methods:**
- `new() -> App` - Creates new App instance with Default impl (src/app/app_impl.rs:30)
- `mode(&self) -> AppMode` - Returns current mode (src/app/app_impl.rs:36)
- `set_mode(&mut self, mode: AppMode)` - Sets mode and handles transitions (src/app/app_impl.rs:40)
- `start_reading(&mut self, text: &str, wpm: u32)` - Starts new reading session (src/app/app_impl.rs:51)
- `advance_reading(&mut self) -> bool` - Auto-advance to next word (src/app/app_impl.rs:62)
- `handle_keypress(&mut self, key: char) -> bool` - Handles keyboard input in Reading mode (src/app/app_impl.rs:89)

### `Theme` (`src/ui/theme.rs:4`)
UI color scheme configuration.
```rust
pub struct Theme {
    pub background: Color,  // #1A1B26 - Midnight background
    pub text: Color,        // #A9B1D6 - Light blue text
    pub anchor: Color,      // #F7768E - Coral red anchor
    pub dimmed: Color,      // #646E96 - Dimmed blue
}
```

**Purpose:** Centralizes color scheme for maintainability. Midnight theme (PRD Section 4.1) with explicit RGB colors.

**Key Methods:**
- `midnight() -> Self` - Returns midnight theme colors (src/ui/theme.rs:18)
- `default() -> Self` - Returns midnight theme (src/ui/theme.rs:27)

### `ReadingState` (`src/reading/state.rs:14`)
Pure reading state with tokens and timing.
```rust
pub struct ReadingState {
    tokens: Vec<Token>,                // Tokenized document
    current_index: usize,              // Current reading position
    wpm: u32,                          // Words per minute setting
    config: ReadingConfig,             // Timing configuration
}
```

**Purpose:** Holds tokenized document, position, and timing state. Pure core logic only.

**Key Methods:**
- `new_with_default_config(tokens: Vec<Token>, wpm: u32) -> Self` - Creates with default config (src/reading/state.rs:27)
- `advance(&mut self)` - Moves to next token (src/reading/state.rs:41)
- `jump_to_next_sentence(&mut self)` - Jumps to next sentence start (src/reading/state.rs:50)
- `jump_to_previous_sentence(&mut self)` - Jumps to previous sentence start (src/reading/state.rs:66)
- `current_token(&self) -> Option<&Token>` - Returns current token (src/reading/state.rs:82)
- `is_at_sentence_start(&self) -> bool` - Checks if at sentence boundary (src/reading/state.rs:89)
- `get_sentence_progress(&self) -> f64` - Returns progress through current sentence (src/reading/state.rs:96)
- `current_token_duration(&self) -> Duration` - Calculates display duration with punctuation/length penalties (src/reading/state.rs:107)
- `get_wpm(&self) -> u32` - Returns current WPM (src/reading/state.rs:117)
- `adjust_wpm(&mut self, delta: i32)` - Adjusts WPM with clamping (src/reading/state.rs:121)

### `Token` (`src/reading/token.rs:1`)
A word with punctuation and metadata.
```rust
pub struct Token {
    pub text: String,                  // The word text
    pub punctuation: Vec<char>,        // Punctuation after word
    pub is_sentence_start: bool,       // Marks sentence boundaries
}
```

**Purpose:** Basic unit for RSVP reading with punctuation and sentence metadata.

### `RsvpRenderer` Trait (`src/rendering/renderer.rs:9`)
Pluggable trait for RSVP rendering backends.
```rust
pub trait RsvpRenderer {
    fn initialize(&mut self) -> Result<(), RendererError>;
    fn render_word(&mut self, word: &str, anchor_position: usize) -> Result<(), RendererError>;
    fn clear(&mut self) -> Result<(), RendererError>;
    fn cleanup(&mut self) -> Result<(), RendererError>;
}
```

**Purpose:** Abstracts rendering implementations (Kitty Graphics, future Sixel/iTerm2). Enables backend switching without changing reading logic. Object-safe trait supporting `Box<dyn RsvpRenderer>`.

**Note:** Removed `supports_subpixel_ovp()` method (no longer needed - all renderers support sub-pixel OVP).

### `WordCache` (`src/rendering/cache.rs:34`)
Word-Level LRU Cache for rendered word buffers to enable consistent 1000+ WPM performance.
```rust
pub struct WordCache {
    cache: LruCache<CacheKey, CachedWord>,  // LRU cache storage
    font_size: f32,                        // Current font size
    hits: u64,                             // Hit counter for telemetry
    misses: u64,                           // Miss counter for telemetry
    total_cached_bytes: u64,               // Memory tracking
    memory_cap_bytes: u64,                 // 75MB default memory cap
}
```

**Public API:**
- `new(capacity) -> Self` - Create new WordCache with specified capacity (src/rendering/cache.rs:60)
- `get_or_render(word, anchor_position, font, metrics) -> Result<CachedWord, CacheError>` - Main cache lookup with automatic rasterization on miss (src/rendering/cache.rs:91)
- `clear()` - Clear cache and reset statistics (src/rendering/cache.rs:136)
- `set_font_size(font_size)` - Update font size and clear cache if changed (src/rendering/cache.rs:144)
- `get_hit_rate() -> f64` - Calculate hits / (hits + misses) **[cfg(test)]** (src/rendering/cache.rs:159)
- `get_memory_usage_mb() -> f64` - Get memory usage in megabytes **[cfg(test)]** (src/rendering/cache.rs:168)

**Cache Key Design:**
- Tuple-based key `(word: String, font_size: f32, anchor_position: usize)` (src/rendering/cache.rs:27)
- Avoids String allocation overhead compared to formatted string keys
- anchor_position is deterministic from `calculate_anchor_position()` (same word = same anchor)

**Performance Characteristics:**
- Cache hit: O(1) lookup (~microseconds)
- Cache miss: O(n) rasterization (~1-5ms)
- Memory cap enforcement: Evicts LRU entries when limit exceeded
- Target hit rate: ~70% with typical English text

### `KittyGraphicsRenderer` (`src/rendering/kitty/mod.rs:20`)
Pixel-perfect RSVP renderer using Kitty Graphics Protocol with sub-pixel OVP anchoring.
```rust
pub struct KittyGraphicsRenderer {
    viewport: Viewport,
    font: Option<FontRef<'static>>,
    font_size: f32,
    font_metrics: Option<FontMetrics>,
    current_image_id: u32,
    word_cache: WordCache,
}
```

**Public API:**
- `new() -> Self` - Create new KittyGraphicsRenderer instance (src/rendering/kitty/mod.rs:32)
- `calculate_font_size_from_cell_height(cell_height_px)` - Calculate font size for reading zone (src/rendering/kitty/mod.rs:42)
- `get_reading_zone_height() -> Option<u32>` - Get reading zone height (total height minus fixed command deck) (src/rendering/kitty/mod.rs:57)
- `get_vertical_center() -> Option<u32>` - Get Y position at 42% of reading zone (src/rendering/kitty/mod.rs:68)
- `viewport() -> &mut Viewport` - Get mutable viewport access (src/rendering/kitty/mod.rs:79)
- `render_bar(word_y, word_height, progress, mode, image_id) -> Result<()>` - Render micro progress bar below word with mode-aware opacity (30% Reading, 100% Paused) (src/rendering/kitty/mod.rs:86)
- `render_macro_gutter(current_word, total_words, reader_area, mode, image_id) -> Result<()>` - Render 4px vertical document progress bar at right edge of reader zone (src/rendering/kitty/mod.rs:120)

**Implements RsvpRenderer trait:**
- `initialize()` - Load font, get metrics, query viewport, init word cache (src/rendering/kitty/mod.rs:108)
- `render_word(word, anchor_position)` - Use word cache for rasterization, transmit via KGP (src/rendering/kitty/mod.rs:139)
- `clear()` - Delete previous image (src/rendering/kitty/mod.rs:211)
- `cleanup()` - Clear word cache, delete all graphics on exit (src/rendering/kitty/mod.rs:218)

**Key Behaviors:**
- Uses embedded JetBrains Mono font via ab_glyph for text rasterization
- Word-Level LRU Cache eliminates redundant rasterization
- Creates RGBA buffer with transparent background (theme handles background)
- Vertical centering at 42% of reading zone height (per PRD Section 4.3)
- Sub-pixel OVP anchoring via positioning module
- Modular design: Decomposed into protocol, rasterizer, and positioning modules

### `protocol` module (`src/rendering/kitty/protocol.rs`)
Kitty Graphics Protocol transmission and encoding functions.

**Public Functions:**
- `encode_image_base64(image) -> String` - Encode RGBA image to base64 (src/rendering/kitty/protocol.rs:14)
- `transmit_graphics(id, width, height, data, x, y) -> io::Result<()>` - Send image via KGP (src/rendering/kitty/protocol.rs:25)
- `delete_image(id) -> io::Result<()>` - Delete specific KGP image (src/rendering/kitty/protocol.rs:45)
- `delete_all_graphics() -> io::Result<()>` - Clear all KGP images (src/rendering/kitty/protocol.rs:52)

### `rasterizer` module (`src/rendering/kitty/rasterizer.rs`)
Word-to-image rasterization using ab_glyph and imageproc.

**Public Functions:**
- `rasterize_word(word, anchor_position, font, font_size, metrics) -> Option<ImageBuffer>` - Render word to RGBA buffer (src/rendering/kitty/rasterizer.rs:21)
- `TEXT_COLOR` - Theme text color constant (#A9B1D6) (src/rendering/kitty/rasterizer.rs:14)
- `ANCHOR_COLOR` - Theme anchor color constant (#F7768E) (src/rendering/kitty/rasterizer.rs:17)

### `positioning` module (`src/rendering/kitty/positioning.rs`)
OVP (Optimal Viewing Position) anchoring calculations.

**Public Functions:**
- `calculate_start_x(word, anchor_position, font, font_size, viewport) -> f32` - Calculate sub-pixel OVP X position (src/rendering/kitty/positioning.rs:18)
- `get_reading_zone_height(viewport) -> Option<u32>` - Calculate reading zone height (src/rendering/kitty/positioning.rs:56)
- `calculate_vertical_center(viewport) -> Option<u32>` - Calculate Y position at 42% of zone (src/rendering/kitty/positioning.rs:71)

### `Viewport` (`src/rendering/viewport.rs:11`)
Viewport coordinate management for graphics overlay pattern.
```rust
pub struct Viewport {
    dimensions: Option<TerminalDimensions>,
}

pub struct TerminalDimensions {
    pixel_size: (u32, u32),  // Total text area in pixels
    cell_count: (u16, u16),  // Total cells (columns, rows)
    cell_size: (f32, f32),   // Size of single cell in pixels
}
```

**Public API:**
- `new() -> Self` - Create new viewport manager (src/rendering/viewport.rs:26)
- `query_dimensions() -> Result<TerminalDimensions, ViewportError>` - Send CSI 14t/18t queries (src/rendering/viewport.rs:30)
- `get_dimensions() -> Option<TerminalDimensions>` - Get current dimensions (src/rendering/viewport.rs:82)

**Removed Methods:**
- ~~`set_dimensions(dimensions)`~~ - Removed (no longer needed)
- ~~`convert_rect_to_pixels(x, y, w, h)`~~ - Removed (simplified API)

**Key Behaviors:**
- Queries terminal using CSI 14t (pixel size) and 18t (cell count)
- Calculates cell dimensions: pixel_size / cell_count
- Enables Viewport Overlay Pattern (PRD Section 4.2)

### `FontMetrics` (`src/rendering/font.rs:12`)
Font metric data for OVP calculations.
```rust
pub struct FontMetrics {
    pub height: f32,      // Total line height
    pub font_size: f32,   // Font size in pixels
}
```

**Purpose:** Holds font metrics for OVP positioning calculations.

**Simplified:** Removed unused fields (`ascent`, `descent`, `line_gap`) during cleanup.

**Public API:**
- `get_font() -> Option<FontRef<'static>>` - Get embedded JetBrains Mono font singleton (src/rendering/font.rs:26)
- `calculate_char_width(font, c, font_size) -> f32` - Calculate character width (src/rendering/font.rs:40)
- `calculate_string_width(font, text, font_size) -> f32` - Calculate string width (src/rendering/font.rs:53)
- `get_font_metrics(font, font_size) -> FontMetrics` - Get font metrics (src/rendering/font.rs:68)

**Removed:**
- ~~`load_font_from_path(path)`~~ - Removed (using embedded font only)
- ~~`get_font_with_config(config)`~~ - Removed (simplified font loading)
- ~~`FontConfig` struct~~ - Removed (not needed)

### `AppMode` (`src/app/mode.rs:1`)
Application operating modes.
```rust
pub enum AppMode {
    Command,   // Command input mode (bottom deck)
    Reading,   // Full-screen TUI reading mode
    Paused,    // Reading mode paused
    Quit,      // Application exit
}
```

**Purpose:** Tracks which UI layer is active and handles transitions.

**Removed:** `Peek` variant (not implemented)

### `Command` (`src/ui/command.rs:10`)
Command deck input variants.
```rust
pub enum Command {
    Quit,                   // :q or :quit
    Help,                   // :h or :help
    LoadFile(String),       // @filename.pdf or @filename.epub
    LoadClipboard,          // @@
    Unknown(String),        // Parse error
}
```

**Purpose:** Parsed command deck input for processing.

**Public Functions:**
- `parse(input: &str) -> Self` - Parse command string into Command enum (src/ui/command.rs:22)

---

## 3. Public Methods

### TuiManager (`src/ui/terminal.rs:23`)
Terminal UI manager with auto-advancement event loop.
```rust
pub struct TuiManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}
```

**Purpose:** Manages TUI mode with word auto-advancement based on WPM timing.

**Key Methods:**
- `new() -> Result<Self, io::Error>` - Creates TUI manager, enables raw mode, enters alternate screen (src/ui/terminal.rs:29)
- `run_event_loop(&mut self, app: &mut App) -> io::Result<AppMode>` - Main event loop with WPM-based auto-advancement (src/ui/terminal.rs:39)
- `render_frame(&mut self, app: &App) -> io::Result<()>` - Renders word display with OVP anchoring (src/ui/terminal.rs:89)

### CommandExecutor (`src/ui/command_executor.rs:10`)
Command execution logic extracted from terminal event loop.

**Public Enum:**
```rust
pub enum CommandResult {
    Continue,           // Continue running, no mode change
    Exit(AppMode),      // Exit event loop with specified mode
}
```

**Public Function:**
- `execute_command(app: &mut App, command_str: &str) -> io::Result<CommandResult>` - Parse and execute command (src/ui/command_executor.rs:18)

**Supported Commands:**
- `LoadFile(path)` - Load PDF/EPUB file
- `LoadClipboard` - Load from clipboard
- `Quit` - Exit application
- `Help` - Show help (placeholder)
- `Unknown` - Show error for invalid commands

### Key Binding Reference (`src/app/app_impl.rs:89`)

Reading mode key bindings (j=left, k=right for VIM-like navigation):
- `'j'` - jump to previous sentence (j is left on keyboard)
- `'k'` - jump to next sentence (k is right on keyboard)
- `'['` - decrease WPM by 50
- `']'` - increase WPM by 50
- `' '` - toggle pause
- `'q'` - quit to Command mode

---

## 4. Module Architecture

### Pure Core Pattern
The project follows **pure core + thin IO adapter** pattern:

1. **Engine Layer** (`src/engine/`) - Pure logic, no I/O
   - Token processing, timing calculations, state transitions
   - Can be tested without terminal or filesystem

2. **App Layer** (`src/app/`) - State coordination
   - Manages mode transitions
   - Coordinates between TUI and engine
   - Delegates to engine for pure logic

3. **IO Adapters** (`src/ui/`, `src/input/`) - I/O wrappers
   - Command parsing in command deck
   - File format parsing (PDF, EPUB, clipboard)
   - TUI rendering (ratatui-based, with OVP anchoring)
   - Theme configuration (centralized color schemes)

### Testing Strategy
- **Unit tests** in `engine/` modules (pure logic)
- **Integration tests** in `tests/` directory (6 tests)
- **Manual TUI testing** required for UI components

---

## 5. Current Implementation Status

### ✅ Implemented (As of 2026-02-09)

**Core Features:**
- ✅ PDF/EPUB/clipboard parsing (`src/input/`)
- ✅ OVP anchor position calculation (`src/reading/ovp.rs`)
- ✅ WPM adjustment ([ / ] keys) (src/app/app_impl.rs:99)
- ✅ Pause/resume (space key) (src/app/app_impl.rs:103)
- ✅ Mode management (Command/Reading/Paused/Quit) (src/app/mode.rs)
- ✅ TUI rendering layer (`src/ui/terminal.rs`, `src/ui/reader/view.rs`)
- ✅ Midnight theme colors (src/ui/theme.rs)
- ✅ Auto-advancement timing loop (src/ui/terminal.rs:39)
- ✅ Sentence-aware navigation (j/k keys) (src/app/app_impl.rs:93)

**Word-Level LRU Cache:**
- ✅ WordCache struct with LRU storage (src/rendering/cache.rs)
- ✅ Tuple-based cache keys for performance
- ✅ Memory tracking with 75MB cap
- ✅ Hit/miss counters for telemetry
- ✅ Integration with KittyGraphicsRenderer
- ✅ Cache used in render_word() for transparent caching

**Image-Based Rendering:**
- ✅ Text rasterization using ab_glyph + imageproc
- ✅ Pixel-perfect RGBA buffer creation with theme colors
- ✅ Sub-pixel OVP anchoring via `calculate_start_x()`
- ✅ Vertical centering at 42% of reading zone
- ✅ Kitty Graphics Protocol transmission
- ✅ Micro progress bar (2px horizontal sentence progress)
- ✅ Macro gutter (4px vertical document progress)
- ✅ Mode-aware opacity (30% Reading / 100% Paused) for all progress indicators

**UI Features:**
- ✅ WPM display in reading zone (top-left corner)
- ✅ Real-time WPM adjustment with [ / ] keys
- ✅ Pause/Resume with Space

**Cleanup Completed:**
- ✅ Removed dead code (~1,700 lines)
- ✅ Consolidated App struct (removed event.rs, render_state.rs)
- ✅ Removed capability.rs (terminal detection)
- ✅ Removed cell.rs (replaced with direct rendering)
- ✅ Simplified FontMetrics (removed unused fields)
- ✅ 0 clippy warnings
- ✅ 138 tests passing (132 unit + 6 integration)

---

## 6. PRD Alignment

| PRD Section | Implementation Status |
|-------------|----------------------|
| **3.1 OVP Anchoring** | ✅ Implemented (`calculate_anchor_position()`, sub-pixel positioning) |
| **3.2 Weighted Delay** | ✅ Complete (punctuation multipliers, length penalty) |
| **3.3 Sentence Navigation** | ✅ Implemented (j=left/k=right keys) |
| **4.1 Midnight Theme** | ✅ Implemented (theme.rs with explicit RGB colors) |
| **4.2 Dual-Engine** | ✅ RsvpRenderer trait with KittyGraphicsRenderer |
| **4.4 Progress Bars** | ✅ Both micro-bar and macro-gutter implemented with mode-aware opacity |
| **7.2 Reading Mode** | ✅ Complete (TUI with OVP anchoring, auto-advance) |

---

## 7. Dependencies

### Core Crates
- `ratatui = "0.30"` - TUI framework ✅
- `crossterm = "0.29"` - Terminal I/O ✅
- `ab_glyph = "0.2.32"` - Font parsing and metrics ✅
- `lazy_static = "1.5"` - Font singleton ✅
- `rustyline = "17.0"` - Command input ✅
- `pdf-extract = "0.10.0"` - PDF parsing ✅
- `epub = "2.1.5"` - EPUB parsing ✅
- `arboard = "3.6.1"` - Clipboard access ✅
- `lru = "0.12"` - LRU cache implementation ✅
- `image = "0.25"` - Image buffer types ✅
- `imageproc = "0.25"` - Image manipulation ✅
- `base64 = "0.22"` - Base64 encoding for KGP ✅

### Development
- `cargo test` - Unit and integration tests (138 passing)
- `cargo clippy` - Linting (0 warnings)
- `cargo fmt` - Code formatting

---

## 8. Key Design Decisions

### 1. TUI-First Command Deck Architecture

- **Command Deck (Bottom section):** Command area for input
- Commands typed directly (no prompt like `speedy>`)
- Reading Zone (Top - dynamic): Displays RSVP content or instructions
- Mode transitions: Command ↔ Reading ↔ Paused
- `:q` in Command Mode exits application entirely

### 2. Simplified App Architecture

- Removed `AppEvent` enum and event handling system
- App now uses direct method calls for state management
- Default impl for App reduces boilerplate
- Simplified mode transitions

### 3. Embedded Font Strategy

- JetBrains Mono bundled via `include_bytes!`
- Removed filesystem font loading (simplified API)
- Single font weight (~300KB) for English text
- No font configuration needed

### 4. LRU Cache Performance

- Word-level caching eliminates redundant rasterization
- 75MB memory cap with automatic eviction
- ~70% hit rate with typical English text
- Enables 1000+ WPM performance

---

## 9. Recent Cleanup

### 2026-02-10
- `src/rendering/progress_bar.rs` - Removed unused module (functionality inlined in kitty/mod.rs)
- Updated PRD to reflect actual implementation status

### 2026-02-09
- `src/engine/error.rs` - Unused error enum
- `src/app/event.rs` - Unused event handling
- `src/app/render_state.rs` - Consolidated into App
- `src/rendering/capability.rs` - Terminal detection (not needed)
- `src/rendering/cell.rs` - Replaced with direct rendering
- `tests/cache_integration.rs` - Consolidated into main tests

### Removed Features
- `AppMode::Peek` - Not implemented
- `GraphicsCapability` enum - Terminal detection removed
- `supports_subpixel_ovp()` - All renderers support it
- FontMetrics `ascent`, `descent`, `line_gap` - Unused
- `load_font_from_path()` - Using embedded font only
- `cell_to_pixel()`, `rect_to_pixel()` - Simplified viewport API

### Net Result
- ~1,700 lines removed
- 0 clippy warnings
- 138 tests passing
- Cleaner, more maintainable codebase

---

## 10. Update Workflow

### When to Update This Document
1. **Adding new public methods** to existing structs
2. **Creating new modules** or files
3. **Changing architecture** patterns
4. **Adding significant dependencies**
5. **Completing major features** (update status)

### When NOT to Update
1. **Test-only changes**
2. **Private method additions**
3. **Refactors without API changes**
4. **Bug fixes** (unless architecture impacted)

### Update Process
1. **After pre-commit validation passes**
2. **Before final git commit**
3. **Document using `file_path:line_number` references**
4. **Keep descriptions brief and factual**

---

**Document Maintainer:** Development agents  
**Verification Method:** Cross-reference with actual codebase using `serena_search_for_pattern`
