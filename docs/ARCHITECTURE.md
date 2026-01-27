# Speedy Architecture Document

**Last Updated:** 2026-01-27  
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
│   ├── timing.rs       # Token struct and timing calculations
│   └── mod.rs          # Engine module exports
├── ui/                 # UI rendering layer (planned, minimal exists)
│   ├── reader.rs        # REPL input handling
│   └── mod.rs          # UI module exports (empty)
├── repl/               # REPL-specific code
│   ├── input.rs        # ReplInput enum and parsing
│   └── mod.rs          # REPL module exports
├── input/              # File input processing
│   ├── pdf.rs          # PDF parsing
│   ├── epub.rs         # EPUB parsing
│   ├── clipboard.rs    # Clipboard content extraction
│   └── mod.rs          # Input module exports
└── main.rs             # Entry point with REPL loop
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
- `'j'/'J'` - jump to next sentence
- `'k'/'K'` - jump to previous sentence  
- `'['` - decrease WPM by 50
- `']'` - increase WPM by 50
- `' '` - toggle pause
- `'q'/'Q'` - quit to REPL

#### Missing Methods (Need Implementation)
- `advance_reading()` - Auto-advance to next word (required for TUI timing loop)

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
   - TUI rendering (planned)

### Testing Strategy
- **Unit tests** in `engine/` modules (pure logic)
- **Integration tests** in `tests/` directory
- **Manual TUI testing** required for UI components

---

## 5. Current Implementation Status

### ✅ Implemented
- REPL with rustyline (`@filename`, `@@`, `:q`, `:h`)
- PDF/EPUB/clipboard parsing
- Sentence-aware navigation (j/k keys)
- WPM adjustment ([ / ] keys)
- Pause/resume (space key)
- Mode management (Repl/Reading/Paused)

### ❌ Missing (Task 2B-1)
- OVP anchoring calculation (`calculate_anchor_position()`)
- TUI rendering layer (`src/ui/render.rs`)
- TUI terminal manager (`src/ui/terminal.rs`)
- `App::advance_reading()` method

### 🚧 In Progress
- TUI integration (Task 2B-1)
- Auto-advancement timing loop

---

## 6. PRD Alignment

| PRD Section | Implementation Status |
|-------------|----------------------|
| **3.1 OVP Anchoring** | ❌ Missing (`calculate_anchor_position()`) |
| **3.2 Weighted Delay** | ✅ Partial (`current_token_duration()`) |
| **3.3 Sentence Navigation** | ✅ Implemented (j/k keys) |
| **4.1 Midnight Theme** | ❌ Missing (TUI rendering) |
| **7.1 REPL Mode** | ✅ Complete |
| **7.2 Reading Mode** | ✅ Partial (needs TUI) |

---

## 7. Dependencies

### Core Crates
- `ratatui = "0.30"` - TUI framework (not yet used)
- `crossterm = "0.29"` - Terminal I/O (not yet used)
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
1. **Missing `advance_reading()`** - Required for auto-advancement timing
2. **No TUI rendering** - Need `render.rs` and `terminal.rs`
3. **No OVP calculation** - Need `calculate_anchor_position()`

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