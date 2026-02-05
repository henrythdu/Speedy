# Speedy Architecture Document

**Last Updated:** 2026-02-04 (Cleanup: Removed CellRenderer references, deleted failing test, updated test count, noted CPU compositing pivot)
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
│   ├── app.rs          # Main App struct and business logic
│   ├── event.rs        # AppEvent enum for event handling
│   ├── mode.rs         # AppMode enum (Repl, Reading, Paused, Command)

│   └── mod.rs          # App module exports
├── engine/             # Shared logic (config, errors, re-exports)
│   ├── config.rs       # ReadingConfig timing configuration
│   ├── error.rs        # SpeedyError enum
│   └── mod.rs          # Engine module (re-exports from reading/ and rendering/)
├── reading/            # Core RSVP reading logic domain
│   ├── token.rs        # Token struct
│   ├── timing.rs       # Tokenization, WPM calculations, sentence boundaries
│   ├── state.rs        # ReadingState with navigation and timing
│   ├── ovp.rs          # OVP anchor position calculation
│   └── mod.rs          # Reading module exports
├── rendering/          # Rendering backends domain

│   ├── renderer.rs     # RsvpRenderer trait and RendererError
│   ├── viewport.rs     # Viewport coordinates and terminal dimensions
│   ├── font.rs         # Font loading and metrics
│   ├── capability.rs   # Terminal capability detection
│   ├── kitty/          # Kitty Graphics Protocol modules
│   │   ├── mod.rs      # KittyGraphicsRenderer implementation
│   │   ├── protocol.rs # KGP transmission and encoding
│   │   ├── rasterizer.rs # Word-to-image rendering
│   │   └── positioning.rs # OVP anchoring calculations
│   └── mod.rs          # Rendering module exports
├── ui/                 # TUI rendering layer
│   ├── reader/         # Reader feature module
│   │   ├── component.rs # ReaderComponent (placeholder for future use)
│   │   └── view.rs     # Render functions (OVP word, progress)
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
└── main.rs             # Entry point with capability detection and TUI launch
```

---

## 2. Core Structs

### `App` (`src/app/app.rs:18`)
Main application state container.
```rust
pub struct App {
    mode: AppMode,                     // Current mode (Repl/Reading/Paused)
    reading_state: Option<ReadingState>, // Current reading session
}
```

**Purpose:** Coordinates between REPL, TUI, and engine layers. Manages mode transitions.

### `Theme` (`src/ui/theme.rs:4`)
UI color scheme configuration.
```rust
pub struct Theme {
    pub background: Color,
    pub text: Color,
    pub anchor: Color,
    pub dimmed: Color,
}
```

**Purpose:** Centralizes color scheme for maintainability. Midnight theme (PRD Section 4.1) with explicit RGB colors to ensure dimmed modifier works correctly.

### `ReadingState` (`src/reading/state.rs:1`)
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

### `RsvpRenderer` Trait (`src/rendering/renderer.rs:37`)
Pluggable trait for RSVP rendering backends.
```rust
pub trait RsvpRenderer {
    fn initialize(&mut self) -> Result<(), RendererError>;
    fn render_word(&mut self, word: &str, anchor_position: usize) -> Result<(), RendererError>;
    fn clear(&mut self) -> Result<(), RendererError>;
    fn supports_subpixel_ovp(&self) -> bool;
    fn cleanup(&mut self) -> Result<(), RendererError>;
}
```

**Purpose:** Abstracts rendering implementations (Kitty Graphics, future Sixel/iTerm2). Enables backend switching without changing reading logic. Object-safe trait supporting `Box<dyn RsvpRenderer>`.

### `KittyGraphicsRenderer` (`src/rendering/kitty/mod.rs:18`)
Pixel-perfect RSVP renderer using Kitty Graphics Protocol with sub-pixel OVP anchoring.
```rust
pub struct KittyGraphicsRenderer {
    viewport: Viewport,
    font: Option<FontRef<'static>>,
    font_size: f32,
    font_metrics: Option<FontMetrics>,
    current_image_id: u32,
}
```

**Public API:**
- `new() -> Self` - Create new KittyGraphicsRenderer instance
- `calculate_font_size_from_cell_height(cell_height_px)` - Calculate font size for 5-line height
- `get_reading_zone_height() -> Option<u32>` - Get reading zone height (total height minus fixed 5-line command deck)
- `calculate_vertical_center() -> Option<u32>` - Calculate Y position at 42% of reading zone
- `viewport() -> &mut Viewport` - Get mutable viewport access

**Implements RsvpRenderer trait:**
- `initialize()` - Load font, get metrics, query viewport
- `render_word(word, anchor_position)` - Rasterize and transmit word via KGP
- `clear()` - Delete previous image
- `cleanup()` - Delete all graphics on exit
- `supports_subpixel_ovp()` - Returns true

**Key Behaviors:**
- Uses embedded JetBrains Mono font via ab_glyph for text rasterization
- Creates RGBA buffer with transparent background (theme handles background)
- Vertical centering at 42% of reading zone height (per PRD Section 4.3)
- Sub-pixel OVP anchoring via positioning module
- Implements RsvpRenderer trait for pluggable backend architecture
- **Modular Design:** Decomposed into protocol, rasterizer, and positioning modules

### `protocol` module (`src/rendering/kitty/protocol.rs`)
Kitty Graphics Protocol transmission and encoding functions.

**Public Functions:**
- `encode_image_base64(image) -> String` - Encode RGBA image to base64
- `transmit_graphics(id, width, height, data, x, y) -> io::Result<()>` - Send image via KGP
- `delete_image(id) -> io::Result<()>` - Delete specific KGP image
- `delete_all_graphics() -> io::Result<()>` - Clear all KGP images

### `rasterizer` module (`src/rendering/kitty/rasterizer.rs`)
Word-to-image rasterization using ab_glyph and imageproc.

**Public Functions:**
- `rasterize_word(word, anchor_position, font, font_size, metrics) -> Option<ImageBuffer>` - Render word to RGBA buffer
- `TEXT_COLOR` - Theme text color constant (#A9B1D6)
- `ANCHOR_COLOR` - Theme anchor color constant (#F7768E)

### `positioning` module (`src/rendering/kitty/positioning.rs`)
OVP (Optimal Viewing Position) anchoring calculations.

**Public Functions:**
- `calculate_start_x(word, anchor_position, font, font_size, viewport) -> f32` - Calculate sub-pixel OVP X position
- `get_reading_zone_height(viewport) -> Option<u32>` - Calculate reading zone height
- `calculate_vertical_center(viewport) -> Option<u32>` - Calculate Y position at 42% of zone
- Background color is Midnight theme #1A1B26 (deep slate)
- Composites all visual elements (background, word, ghost words) into single buffer
- Eliminates flickering and Z-fighting issues from multiple image transmissions
- Canvas-relative positioning (words at 42% of canvas, not full screen) fixes coordinate bug

### `Viewport` (`src/rendering/viewport.rs:38`)
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
- `new() -> Self` - Create new viewport manager
- `query_dimensions() -> Result<TerminalDimensions, ViewportError>` - Send CSI 14t/18t queries
- `set_dimensions(dimensions)` - Set dimensions manually (for testing)
- `get_dimensions() -> Option<TerminalDimensions>` - Get current dimensions
- `convert_rect_to_pixels(x, y, w, h) -> Option<(u32, u32, u32, u32)>` - Convert Ratatui Rect to pixels

**Key Behaviors:**
- Queries terminal using CSI 14t (pixel size) and 18t (cell count)
- Calculates cell dimensions: pixel_size / cell_count
- Converts Ratatui cell coordinates to pixel coordinates for graphics rendering
- Enables Viewport Overlay Pattern (PRD Section 4.2, Design Doc v2.0 Section 2.1)

### `GraphicsCapability` (`src/rendering/capability.rs:8`)
Terminal graphics support level enum.
```rust
pub enum GraphicsCapability {
    None,   // Terminal does not support Kitty Graphics Protocol (will exit with error)
    Kitty,  // Kitty Graphics Protocol supported
}
```

**Purpose:** Tracks detected terminal capability. Application requires Kitty Graphics Protocol support; exits with clear error if not available.

### `FontMetrics` (`src/rendering/font.rs`)
Font metric data for OVP calculations.
```rust
pub struct FontMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub height: f32,
    pub font_size: f32,
}
```
**Purpose:** Holds font metrics (ascent, descent, line_gap, height) for OVP positioning calculations.

**Public API:**
- `get_font()` -> `Option<FontRef<'static>>` - Get embedded JetBrains Mono font singleton
- `load_font_from_path(path)` -> `Option<FontRef<'static>>` - Load font from filesystem
- `get_font_with_config(config)` -> `Option<FontRef<'static>>` - Config-based font loading
- `calculate_char_width(font, c, font_size)` -> `f32` - Calculate character width
- `calculate_string_width(font, text, font_size)` -> `f32` - Calculate string width
- `get_font_metrics(font, font_size)` -> `FontMetrics` - Get full font metrics
- `FontConfig` - Configuration struct for font loading

**Key Dependencies:** `ab_glyph`, `lazy_static`

### `CapabilityDetector` (`src/rendering/capability.rs:26`)
Terminal capability detection logic.
```rust
pub struct CapabilityDetector;
impl CapabilityDetector {
    pub fn new() -> Self;
    pub fn detect(&self) -> GraphicsCapability;
}
```

**Purpose:** Detects terminal graphics capabilities via environment variables ($TERM). Application requires Kitty Graphics Protocol support; exits with clear error if not available.

### `AppMode` (`src/app/mode.rs:1`)
Application operating modes.
```rust
pub enum AppMode {
    Command,   // Command input mode (bottom deck)
    Reading,   // Full-screen TUI reading mode
    Paused,    // Reading mode paused
    Peek,      // Peek mode (hold Tab to see context)
    Quit,      // Application exit
}
```

**Purpose:** Tracks which UI layer is active and handles transitions.

### `ReaderComponent` (`src/ui/reader/component.rs:9`)
Reader UI component (placeholder for future use).

**Purpose:** Reserved for future UI component architecture.

### `Command` (`src/ui/command.rs:18`)
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

**Purpose:** Parsed command deck input for processing. Replaces the obsolete REPL module.

---

## 3. Public Methods

### App Methods (`src/app/app.rs`)

#### State Management
- `pub fn new() -> App` - Creates new App instance
- `pub fn mode(&self) -> AppMode` - Returns current mode (line 190)
- `pub fn set_mode(&mut self, mode: AppMode)` - Sets mode (line 194)

#### Reading Session
- `pub fn get_wpm(&self) -> u32` - Returns WPM or default 300 (line 198)
- `pub fn resume_reading(&mut self) -> Result<(), String>` - Resumes paused session (line 134)
- `pub fn apply_loaded_document(&mut self, doc: LoadedDocument)` - Applies loaded document
- `pub fn start_reading(&mut self, text: &str, wpm: u32)` - Starts reading session

#### Input Handling
- `pub fn handle_event(&mut self, event: AppEvent)` - Processes app events
- `pub fn handle_keypress(&mut self, key: char) -> bool` - Handles keyboard input in Reading mode (line 227)

**Key binding implementation (handle_keypress):**
- `'j'/'J'` - jump to previous sentence (j is left on keyboard)
- `'k'/'K'` - jump to next sentence (k is right on keyboard)  
- `'['` - decrease WPM by 50
- `']'` - increase WPM by 50
- `' '` - toggle pause
- `'q'/'Q'` - quit to REPL

#### TUI Integration
- `pub fn advance_reading(&mut self) -> bool` - Auto-advance to next word, returns true if advanced (line 51)

### TuiManager (`src/ui/terminal.rs:20`)
Terminal UI manager with auto-advancement event loop.
```rust
pub struct TuiManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}
```

**Purpose:** Manages TUI mode with word auto-advancement based on WPM timing.

**Key Methods:**
- `pub fn new() -> Result<Self, io::Error>` - Creates TUI manager, enables raw mode, enters alternate screen (src/ui/terminal.rs:26)
- `pub fn run_event_loop(&mut self, app: &mut App) -> io::Result<AppMode>` - Main event loop with WPM-based auto-advancement (src/ui/terminal.rs:36)
- `pub fn render_frame(&mut self, app: &App) -> io::Result<()>` - Renders word display with OVP anchoring (src/ui/terminal.rs:78)

**Render Layout:**
- Context left (40%), word display (20%), context right (40%)
- Progress bar at bottom of main area (90% of screen)
- Gutter on far right (3% of screen width)
- OVP anchor position: calculates left padding to keep anchor at visual center (src/ui/reader/view.rs:10)

### CommandExecutor (`src/ui/command_executor.rs`)
Command execution logic extracted from terminal event loop.

**Public Enum:**
- `CommandResult::Continue` - Continue running, no mode change
- `CommandResult::Exit(AppMode)` - Exit event loop with specified mode

**Public Function:**
- `execute_command(app, command_str) -> io::Result<CommandResult>` - Parse and execute command

**Supported Commands:**
- `LoadFile(path)` - Load PDF/EPUB file
- `LoadClipboard` - Load from clipboard
- `Quit` - Exit application
- `Help` - Show help (placeholder)
- `Unknown` - Show error for invalid commands

**Purpose:** Separates command business logic from event loop orchestration, improving testability and SRP compliance.

### ReadingState Methods (`src/reading/state.rs`)

#### Navigation
- `pub fn advance(&mut self)` - Moves to next token (line 83)
- `pub fn jump_to_next_sentence(&mut self)` - Jumps to next sentence start
- `pub fn jump_to_previous_sentence(&mut self)` - Jumps to previous sentence start

#### Timing & Configuration
- `pub fn get_wpm(&self) -> u32` - Returns current WPM (line 39)
- `pub fn adjust_wpm(&mut self, delta: i32)` - Adjusts WPM with clamping
- `pub fn current_token_duration(&self) -> Duration` - Calculates token display duration

#### Factory Methods
- `pub fn new_with_default_config(tokens: Vec<Token>, wpm: u32) -> Self` - Creates with default config

### Theme Methods (`src/ui/theme.rs`)
- `pub fn midnight() -> Self` - Returns midnight theme colors
- `pub fn current() -> Self` - Returns default theme (midnight)

### Theme Colors Module (`src/ui/theme.rs:44`)
- `pub fn background() -> Color` - Midnight background (#1A1B26)
- `pub fn text() -> Color` - Light blue text (#A9B1D6)
- `pub fn anchor() -> Color` - Coral red anchor (#F7768E)
- `pub fn dimmed() -> Color` - Dimmed blue (#646E96)

---

## 4. Module Architecture

### Pure Core Pattern
The project follows **pure core + thin IO adapter** pattern:

1. **Engine Layer** (`src/engine/`) - Pure logic, no I/O
   - Token processing, timing calculations, state transitions
   - Can be tested without terminal or filesystem

2. **App Layer** (`src/app/`) - State coordination
   - Manages mode transitions
   - Coordinates between REPL and TUI
   - Delegates to engine for pure logic

3. **IO Adapters** (`src/ui/`, `src/repl/`, `src/input/`) - I/O wrappers
    - Commands parsed via rustyline in command deck
    - File format parsing (PDF, EPUB)
    - TUI rendering (ratatui-based, with OVP anchoring) ✅
    - Theme configuration (centralized color schemes) ✅

### Testing Strategy
- **Unit tests** in `engine/` modules (pure logic)
- **Integration tests** in `tests/` directory
- **Manual TUI testing** required for UI components

---

## 5. Current Implementation Status

### ✅ Implemented (Epic 2 Complete)
- REPL with rustyline (`@filename`, `@@`, `:q`, `:h`)
- PDF/EPUB/clipboard parsing
- OVP anchor position calculation (`calculate_anchor_position()`) (src/reading/ovp.rs:17)
- WPM adjustment ([ / ] keys)
- Pause/resume (space key)
- Mode management (Repl/Reading/Paused/Command/Quit)
- TUI rendering layer (`src/ui/reader/view.rs`, `src/ui/terminal.rs`)
- Midnight theme colors (`src/ui/theme.rs`)
- Auto-advancement timing loop
- OVP anchoring (left padding calculation in render_word_display) (src/ui/reader/view.rs:10)
- ReaderComponent UI wrapper (src/ui/reader/component.rs) - placeholder for future use
- Domain-based organization (reading/ and rendering/ modules)
- Application layer refactoring (app.rs split into event.rs, mode.rs)
- UI layer refactoring (reader/ subdirectory with component.rs and view.rs)

### 🚧 In Progress (Epic 2: Image-Based Word Rendering)

**Composite Rendering Implementation (COMPLETE - Bug Fixed!):**
- ✅ Task 1: ReadingCanvas struct for full-zone composite rendering (src/rendering/kitty.rs:38)
- ✅ Task 2: create_canvas() method (src/rendering/kitty.rs:196)
- ✅ Task 3: composite_word() method with **coordinate bug fix** (src/rendering/kitty.rs:208)
- ✅ Task 4: render_frame() orchestrator (src/rendering/kitty.rs:306)
- ✅ Task 5: TuiManager integration update (src/ui/terminal.rs:206)
- ✅ Task 6: Verification and cleanup - **All 220 tests pass**

**Bug Fixed:** Words now render at 42% of READING ZONE (canvas-relative) instead of 42% of FULL SCREEN (screen-relative). This places words in the middle of the reading area instead of near the command deck.

**Testing:** 178 total tests passing (unit + integration), 0 failures.

**Previously Implemented:**
- ✅ Task 2: ab_glyph Word Rasterization (COMPLETE)
- ✅ Task 3: Kitty Protocol Image Display (COMPLETE)

**Epic 2 Features Implemented:**
- Text rasterization using ab_glyph + imageproc
- Pixel-perfect RGBA buffer creation with theme colors
- Sub-pixel OVP anchoring via `calculate_start_x()`
- Vertical centering at 42% of reading zone
- Kitty Graphics Protocol transmission with position coordinates
- **Note:** CPU Compositing with ReadingCanvas was attempted but pivoted back to single-word rendering for accurate positioning (may revisit in future)

---

## 6. PRD Alignment

| PRD Section | Implementation Status |
|-------------|----------------------|
| **3.1 OVP Anchoring** | ✅ Implemented (`calculate_anchor_position()`, left padding in render) |
| **3.2 Weighted Delay** | ✅ Complete (floating-point timing precision) |
| **3.3 Sentence Navigation** | ✅ Implemented (j=left/k=right keys) |
| **4.1 Midnight Theme** | ✅ Implemented (theme.rs with explicit RGB colors) |
| **4.2 Dual-Engine** | ✅ RsvpRenderer trait with KittyGraphicsRenderer |
| **7.1 REPL Mode** | ✅ Complete |
| **7.2 Reading Mode** | ✅ Complete (TUI with OVP anchoring) |
| **9.2 Terminal Requirements** | ✅ Clear requirement: Kitty Graphics Protocol mandatory (Kitty or Konsole 22.04+) |

---

## 7. Dependencies

### Core Crates
- `ratatui = "0.30"` - TUI framework ✅
- `crossterm = "0.29"` - Terminal I/O ✅
- `ab_glyph = "0.2.32"` - Font parsing and metrics ✅
- `lazy_static = "1.5"` - Font singleton ✅
- `rustyline = "17.0"` - REPL implementation ✅
- `pdf-extract = "0.8"` - PDF parsing ✅
- `epub = "0.3"` - EPUB parsing ✅
- `clipboard = "0.5"` - Clipboard access ✅
- `unicode-segmentation` - Unicode width handling for emoji/CJK (Cargo.toml)

### Development
- `cargo test` - Unit and integration tests
- `cargo clippy` - Linting
- `cargo fmt` - Code formatting

---

## 8. Key Design Decisions

### 1. TUI-First Command Deck Architecture

- **Command Deck (Bottom 5 lines):** Fixed-height command area using rustyline for input
- Commands typed directly (no prompt like `speedy>`)
- Commands execute immediately (similar to OpenCode/Neovim command mode)
- Reading Zone (Top - dynamic): Displays RSVP content or instructions, expands/contracts with terminal size
- Mode transitions: Command ↔ Reading ↔ Paused
- `:q` in Command Mode exits application entirely
- Terminal resize supported: Word re-centers dynamically, auto-pause/resume during resize

**Purpose:** Modern TUI workflow with integrated command input, no REPL prompt

### 2. Full TUI Always-On
- Application launches in full TUI mode immediately (no REPL prompt)
- ReadingState preserved across mode changes
- Last reading position restored if available
- Commands integrated into bottom command deck (rustyline input)

**Purpose:** Modern TUI experience with integrated workflow

### 3. Integrated Command Deck
- Command deck always visible at bottom of TUI
- ReadingState preserved across sessions
- Last reading position restored on app launch
- Quit command (`:q`) exits application entirely

### 4. Integrated Input Handling
- Command deck uses rustyline for command input (like OpenCode command section)
- TUI delegates command parsing to `app.handle_event()`
- Centralized input processing in command deck area

---

## 9. Known Architecture Gaps

### Immediate (Task 2B-1)
1. **Timing precision fix** (Bead 2B-1-0) - REQUIRED BEFORE ANY TUI WORK
2. **Missing `advance_reading()`** (Bead 2B-1-2) - Required for auto-advancement timing
3. **No TUI rendering** (Bead 2B-1-3) - Need `render.rs` and `terminal.rs`
4. **No OVP calculation** (Bead 2B-1-1) - Need `calculate_anchor_position()`

### Future
1. **Audio metronome** (Task 2C-X) - Speed glide, thump sounds
2. **Gutter implementation** (Task 2B-5) - Spatial awareness
3. **Performance optimizations** - Large document handling

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