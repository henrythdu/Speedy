# Speedy Architecture Document

**Last Updated:** 2026-01-28 (Epic 1: RsvpRenderer trait added)  
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
 │   ├── mode.rs         # AppMode enum (Repl, Reading, Paused)
 │   └── mod.rs          # App module exports
 ├── engine/             # Pure core logic (no I/O, no side effects)
 │   ├── state.rs        # ReadingState and token processing
 │   ├── ovp.rs          # OVP anchor position calculation
 │   ├── renderer.rs     # RsvpRenderer trait for pluggable backends
 │   ├── timing.rs       # Token struct and timing calculations
 │   └── mod.rs          # Engine module exports
 ├── ui/                 # TUI rendering layer
 │   ├── render.rs       # Rendering functions (OVP word, progress, context)
 │   ├── terminal.rs     # TuiManager with event loop and frame rendering
 │   ├── theme.rs        # Theme configuration (Midnight colors)
 │   └── mod.rs          # UI module exports
 ├── repl/               # REPL-specific code
 │   ├── input.rs        # ReplInput enum and parsing
 │   └── mod.rs          # REPL module exports
 ├── input/              # File input processing
 │   ├── pdf.rs          # PDF parsing
 │   ├── epub.rs         # EPUB parsing
 │   ├── clipboard.rs    # Clipboard content extraction
 │   └── mod.rs          # Input module exports
 └── main.rs             # Entry point with REPL→TUI transition on Reading mode
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

### `ReadingState` (`src/engine/state.rs:1`)
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

### `Token` (`src/engine/timing.rs:1`)
A word with punctuation and metadata.
```rust
pub struct Token {
    pub text: String,                  // The word text
    pub punctuation: Vec<char>,        // Punctuation after word
    pub is_sentence_start: bool,       // Marks sentence boundaries
}
```

**Purpose:** Basic unit for RSVP reading with punctuation and sentence metadata.

### `RsvpRenderer` Trait (`src/engine/renderer.rs:37`)
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

**Purpose:** Abstracts rendering implementations (TUI CellRenderer, Kitty Graphics, future Sixel/iTerm2). Enables backend switching without changing reading logic. Object-safe trait supporting `Box<dyn RsvpRenderer>`.

### `AppMode` (`src/app/mode.rs:1`)
Application operating modes.
```rust
pub enum AppMode {
    Repl,      // REPL command mode (rustyline)
    Reading,   // Full-screen TUI reading mode
    Paused,    // Reading mode paused
}
```

**Purpose:** Tracks which UI layer is active and handles transitions.

### `ReplInput` (`src/repl/input.rs:15`)
REPL command variants.
```rust
pub enum ReplInput {
    LoadFile(PathBuf),      // @filename
    LoadClipboard,          // @@
    Quit,                    // :q
    Help,                   // :h
    Invalid(String),        // Parse error
}
```

**Purpose:** Parsed REPL commands for processing.

---

## 3. Public Methods

### App Methods (`src/app/app.rs`)

#### State Management
- `pub fn new() -> App` - Creates new App instance
- `pub fn mode(&self) -> AppMode` - Returns current mode (line 190)
- `pub fn set_mode(&mut self, mode: AppMode)` - Sets mode (line 194)

#### Reading Session
- `pub fn get_wpm(&self) -> u32` - Returns WPM or default 300 (line 198)
- `pub fn get_render_state(&self) -> RenderState` - Gets TUI rendering data (line 143)
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
- `pub fn new() -> Result<Self, io::Error>` - Creates TUI manager, enables raw mode, enters alternate screen (line 25)
- `pub fn run_event_loop<F>(&mut self, app: &mut App, render_frame: F) -> io::Result<AppMode>` - Main event loop with WPM-based auto-advancement (line 35)
- `pub fn render_frame(&mut self, app: &App) -> io::Result<()>` - Renders word display with OVP anchoring (line 74)

**Render Layout:**
- Context left (40%), word display (20%), context right (40%)
- Progress bar at bottom of main area (90% of screen)
- Gutter on far right (3% of screen width)
- OVP anchor position: calculates left padding to keep anchor at visual center (src/ui/render.rs:13)

### ReadingState Methods (`src/engine/state.rs`)

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
    - REPL input parsing
    - File format parsing (PDF, EPUB)
    - TUI rendering (ratatui-based, with OVP anchoring) ✅
    - Theme configuration (centralized color schemes) ✅

### Testing Strategy
- **Unit tests** in `engine/` modules (pure logic)
- **Integration tests** in `tests/` directory
- **Manual TUI testing** required for UI components

---

## 5. Current Implementation Status

### ✅ Implemented
- REPL with rustyline (`@filename`, `@@`, `:q`, `:h`)
- PDF/EPUB/clipboard parsing
- OVP anchor position calculation (`calculate_anchor_position()`) (src/engine/ovp.rs:17)
- WPM adjustment ([ / ] keys)
- Pause/resume (space key)
- Mode management (Repl/Reading/Paused/Quit)
- TUI rendering layer (`src/ui/render.rs`, `src/ui/terminal.rs`)
- Midnight theme colors (`src/ui/theme.rs`)
- Auto-advancement timing loop
- OVP anchoring (left padding calculation in render_word_display) (src/ui/render.rs:10)

### ❌ Missing
- None (Task 2B-1 complete)

### 🚧 In Progress
- Performance optimization: Reduce Vec<Token> cloning in get_render_state (src/app/app.rs:143)

---

## 6. PRD Alignment

| PRD Section | Implementation Status |
|-------------|----------------------|
| **3.1 OVP Anchoring** | ✅ Implemented (`calculate_anchor_position()`, left padding in render) |
| **3.2 Weighted Delay** | ✅ Complete (floating-point timing precision) |
| **3.3 Sentence Navigation** | ✅ Implemented (j=left/k=right keys) |
| **4.1 Midnight Theme** | ✅ Implemented (theme.rs with explicit RGB colors) |
| **7.1 REPL Mode** | ✅ Complete |
| **7.2 Reading Mode** | ✅ Complete (TUI with OVP anchoring) |

---

## 7. Dependencies

### Core Crates
- `ratatui = "0.30"` - TUI framework ✅
- `crossterm = "0.29"` - Terminal I/O ✅
- `rustyline = "17.0"` - REPL implementation ✅
- `pdf-extract = "0.8"` - PDF parsing ✅
- `epub = "0.3"` - EPUB parsing ✅
- `clipboard = "0.5"` - Clipboard access ✅

### Development
- `cargo test` - Unit and integration tests
- `cargo clippy` - Linting
- `cargo fmt` - Code formatting

---

## 8. Key Design Decisions

### 1. REPL-First Architecture
- REPL remains primary interface
- TUI launched as modal session from REPL
- Users can load files, check help, then enter reading mode

### 2. Pure Core Separation
- Engine layer has no I/O dependencies
- Enables comprehensive unit testing
- Clear separation of concerns

### 3. State Preservation
- ReadingState preserved on `q` command
- `r` command resumes without reloading
- Enables multi-document workflow

### 4. Delegated Key Handling
- TUI delegates to `app.handle_keypress()`
- Avoids duplicate key binding logic
- Centralized input processing

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