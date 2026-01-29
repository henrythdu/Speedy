---

# 🚀 SPEEDY: MASTER PRODUCT REQUIREMENTS DOCUMENT

**Version:** 2.0  
**Last Updated:** 2026-01-28

**Project Intent:** A terminal RSVP reader that serves as a pacing and focus tool through foveal anchoring and spatial awareness. Uses a **Dual-Engine Architecture** combining traditional TUI for commands with pixel-perfect graphics rendering for reading.

---

## 1. VISION & SCIENTIFIC FOUNDATION

**Speedy** is built on the principle that speed reading is a matter of **Ocular Efficiency** and **Cognitive Load Management**.

* **OVP (Optimal Viewing Position):** Based on *O'Regan & Lévy-Schoen (1987)*, eye saccades account for ~10% of reading time. RSVP enforces consistent pacing and reduces cognitive friction of eye movement.
* **Modality Effect:** Using auditory cues to provide a temporal metronome for the visual stream.

---

## 2. INPUT MODEL (CLI WORKFLOW)

**Speedy** works from any directory, using intuitive file tagging similar to modern CLI tools like Claude Code.

### 2.1 Launch Patterns

**Speedy** starts in full-screen TUI mode with an integrated command deck at the bottom (similar to OpenCode's command section).

**Command Deck Input:**
- Type commands directly into command area (no `speedy>` prompt)
- Commands appear and are executed immediately
- Similar to VS Code/Neovim command mode

**Supported Commands:**
- `@filename.pdf` or `@filename.epub` → Load file (PDF/EPUB)
- `@@` → Load clipboard contents
- `:q` or `:quit` → Quit application
- `:h` or `:help` → Show help/instructions

**Startup Behavior:**
- App launches directly into full TUI mode
- If previous reading state exists, it's restored
- Command deck is always visible at bottom of screen

### 2.2 Supported Formats (MVP)

| Format | Input Method | Notes |
|--------|--------------|-------|
| **PDF** | `@filename.pdf` | Extract text via `pdf-extract` or `poppler` |
| **EPUB** | `@filename.epub` | Parse via `epub` crate |
| **Clipboard** | `@@` | System clipboard access |

### 2.3 Discovery

- **Tab completion**: File suggestions in current directory
- **Recursive search**: `@**/chapter.pdf` to find in subdirectories
- **Recent files**: `@` alone shows reading history

---

## 3. THE READING ENGINE (ENGINE)

### 3.1 Fixed-Axis OVP Anchoring

Words are horizontally shifted so that **Anchor Letter** remains at a fixed vertical coordinate.

**Anchor Position Formula f(word_length):**
```
anchor_index = match word_length:
    1      → 0    (1st letter)
    2-5    → 1    (2nd letter)
    6-9    → 2    (3rd letter)
    10-13   → 3    (4th letter)
     14+     → 3    (cap at 4th position for MVP; Phase 2: ~33% position)
```

**Sub-Pixel Precision (Kitty Graphics Mode):**
When running on Kitty-compatible terminals (Konsole, Kitty), OVP uses **pixel-perfect positioning** rather than character-grid snapping. The anchor character is mathematically centered at the visual fixation point using sub-pixel glyph metrics.

```
W_prefix = Pixel width of glyphs before anchor index
W_anchor_center = Half of pixel width of anchor glyph
StartX = (CanvasWidth/2) - (W_prefix + W_anchor_center)
```

* **Salience:** Anchor is colored `#F7768E` (Coral Red) and pulses in luminance at paragraph breaks.

### 3.2 Weighted Delay Algorithm (Non-Linear Timing)

Instead of a static WPM, time-per-word is calculated as:

**Base Delay Formula:**
```
base_delay_ms = 60000 / wpm
```

**Punctuation Multipliers (stacking rule: max of all applicable):**
* `.` (3.0x), `,` (1.5x), `?` (3.0x), `!` (3.0x), `\n` (4.0x)
* If word has multiple punctuation types (e.g., "word?!"), apply the **maximum** multiplier only
* Delay per word: `delay_ms = base_delay_ms * max(multipliers)`

**Word Length Penalty:**
* Words >10 characters apply configurable delay penalty (default 1.15x, user-adjustable)
* Final delay: `delay_ms = delay_ms * word_length_penalty_if_applicable`

**Chunking:** Common 2-letter pairs (e.g., "in it") are flashed together.

### 3.3 Sentence-Aware Navigation

Navigation jumps (`j`/`k`) always land at sentence beginnings to prevent users from starting mid-sentence, which disrupts reading comprehension.

**Sentence Boundary Rules:**
* Terminal punctuation marks: `.`, `?`, `!` indicate sentence ends
* Newlines indicate sentence boundaries
* Common abbreviations do NOT break sentences: `Dr.`, `Mr.`, `Mrs.`, `Ms.`, `St.`, `Jr.`, `e.g.`, `i.e.`, `vs.`, `etc.`
* Decimal numbers do NOT break sentences: `3.14`, `2.5`, `1,000` (period after number is not sentence terminator)

* **Backward (`k`):** Find the nearest sentence start before current position
* **Forward (`j`):** Find the next sentence start after current position

---

## 4. VISUAL ERGONOMICS (UI/UX)

### 4.1 The "Midnight" Theme

Designed to meet **WCAG AA accessibility** while minimizing eye strain.

| Color | Hex | Purpose |
|-------|-----|---------|
| **Background** | `#1A1B26` (Stormy Dark) | Base canvas |
| **Text** | `#A9B1D6` (Light Blue) | Primary focus - **7.55:1 contrast** (WCAG AA/AAA) |
| **Anchor** | `#F7768E` (Coral Red) | OVP fixation point |
| **Ghost** | `#646E96` (Dimmed Blue) | Context words at 15% opacity |
| **Surface** | `#24283B` (Dark Slate) | Command deck background |

### 4.2 Dual-Engine Architecture

**Overview:** Speedy uses two distinct rendering engines:

1. **Command Layer (Ratatui):** Standard character-grid TUI for REPL input, progress bars, and UI chrome
2. **Reading Layer (Graphics Engine):** Pixel-perfect rendering for RSVP display with sub-pixel OVP anchoring

**Viewport Overlay Pattern:**
- Ratatui renders the overall layout (borders, command section, progress bars)
- Reserves an empty placeholder block for the reading zone
- Graphics engine calculates pixel coordinates from Ratatui's placeholder
- Graphics engine writes directly to terminal using graphics protocol escape sequences (bypassing Ratatui buffer)

**Rendering Modes:**

| Mode | Visual State | Backend | Features |
|------|--------------|---------|----------|
| **Command** | Command Bright / Reader Dimmed (30%) | Ratatui | Standard TUI |
| **Reading** | Reader Full / Command Dimmed (10%) | KittyGraphicsRenderer | Sub-pixel OVP, true opacity |
| **Paused** | Reader Full / Gutter Highlighted | KittyGraphicsRenderer | Full features, paused state |
| **Fallback** | Standard TUI | CellRenderer | Character-grid OVP, dim attribute |

### 4.3 The "Anchor" System (Pixel-Perfect)

The "Anchor" letter (Optimal Recognition Point) remains **mathematically stationary** to eliminate eye jitter.

**Three-Container Model:**
- **Ghost Left:** Previous word, right-aligned, **15% opacity**
- **Focus Center:** Current word, Anchor centered on ORP, **100% opacity**
- **Ghost Right:** Next word, left-aligned, **15% opacity**

**Visual Layout:**
- Reader Zone occupies **top 85%** of terminal
- Reading line centered at **42%** of Reader Zone height
- **10% horizontal padding** inside canvas to buffer word cluster
- Command Section occupies **bottom 15%** (~5 lines)
- **1-line transparent gutter** separates Reader from Command

### 4.4 Progress & Spatial Awareness

**Macro-Gutter (Document Depth):**
- **Visual:** 4px vertical bar on extreme right of Reader Zone
- **Fill Logic:** Top-to-Bottom fill representing `Current Word Index / Total Words`
- **UX Detail:** Dims during Reading Mode (20% opacity); brightens during Pause Mode (100% opacity)

**Micro-Bar (Sentence Context):**
- **Visual:** 2px high horizontal bar, 10px below center word
- **Length:** 25% to 75% of center container width
- **Fill Logic:** Left-to-Right representing progress through current sentence
- **Style:** Completed = `Theme.fg`, Unread = `Theme.ghost`

**Fallback Mode (TUI-only):**
When running on terminals without Kitty Graphics Protocol support:
- OVP snaps to **nearest character cell** (no sub-pixel positioning)
- Ghost words use standard **dim attribute** (not true opacity)
- Progress bars use standard **Unicode block characters**
- Fully functional as RSVP reader with reduced visual fidelity

---

## 5. AUDITORY & KINESTHETIC LAYERS

### 5.1 Auditory Metronome (User-Configurable)

* **Paragraph "Thump":** Low-frequency (100Hz default, range: 80-120Hz) pulse on context shifts.
* **Speed Glide:** Sine wave glide (440Hz → 880Hz) when increasing WPM.
* **Isolation:** Optional subliminal Brown Noise layer.
* **Profiles:** Preset audio profiles (Minimal/Subtle/Pronounced) available in settings.

### 5.2 Tactile Controls

* **Tab-Peek:** Holding `Tab` pauses RSVP and reveals standard text view (Spatial Constancy).
* **Tactical Throttle:** `Shift` drops speed to 50% for dense technical sections.
* **Ocular Priming:** 5-second ramp-up from 70% to 100% WPM on resume.

---

## 6. TECHNICAL ARCHITECTURE

### 6.1 Project Structure (Rust)

```text
speedy/
├── assets/             # JetBrains Mono font, config.toml (Embedded via include_bytes!)
├── src/
│   ├── input/          # pdf.rs, epub.rs, clipboard.rs (File parsing)
│   ├── engine/         # timing.rs, ovp.rs (Word positioning, delay logic)
│   ├── graphics/       # NEW: Rendering backends
│   │   ├── mod.rs      # RsvpRenderer trait definition
│   │   ├── kitty.rs    # KittyGraphicsRenderer (Kitty Protocol)
│   │   ├── cell.rs     # CellRenderer (TUI fallback)
│   │   ├── cache.rs    # Word-Level LRU cache
│   │   └── compositor.rs # CPU compositing (single buffer)
│   ├── ui/             # theme.rs, render.rs (Ratatui TUI layer)
│   ├── storage/        # history.rs (Recent files, reading position)
│   ├── app.rs          # State Machine (AppMode Enum)
│   └── main.rs         # Event Loop & REPL
```

### 6.2 Distribution Strategy

* **Single Binary:** Assets embedded in code; self-initializing config in `~/.config/speedy/`.
* **CI/CD:** GitHub Actions for automated binary releases (Mac/Linux/Windows).
* **Install:** `cargo install speedy-rs`.

### 6.3 Technical Implementation Notes

**Performance Requirements (1000+ WPM):**
- **Per-frame budget:** <10ms total
- **Rasterization (cache hit):** <0.5ms
- **Rasterization (cache miss):** <3ms
- **Encoding + transmission:** <7ms
- **Word-Level LRU Cache:** 1000 entries minimum, ~70% hit rate
- **CPU Compositing:** Single buffer with alpha blending (no multi-layer flicker)

**Accessibility Compliance:**
- All functional text meets WCAG AA contrast ratio (≥4.5:1)
- TUI fallback ensures basic functionality on all terminals

**Font Management:**
- Bundled JetBrains Mono OTF (~300KB) via `include_bytes!`
- Optional `font_path` config override for user preferences
- License: Apache 2.0 (included in `/licenses/` directory)

### 6.4 Pluggable Graphics Backend

**RsvpRenderer Trait:**
```rust
pub trait RsvpRenderer {
    fn render_word(&mut self, current: &str, prev: Option<&str>, 
                   next: Option<&str>, area: Rect) -> Result<()>;
    fn clear(&mut self) -> Result<()>;
    fn supports_subpixel_ovp(&self) -> bool;
}
```

**Implementations:**
- `KittyGraphicsRenderer`: Full pixel-perfect rendering with sub-pixel OVP
- `CellRenderer`: TUI fallback with character-grid OVP snapping

**Future Backends:**
- `SixelRenderer`: Sixel graphics protocol support
- `ITerm2Renderer`: iTerm2 inline image protocol support

**Viewport Overlay Pattern:**
1. Ratatui renders layout and reserves placeholder for reading zone
2. Extract absolute coordinates from Ratatui's placeholder
3. Graphics engine writes pixels directly using protocol escape sequences
4. Avoids forcing pixel rendering into Ratatui's cell-grid workflow

---

## 7. KEYBINDINGS

### 7.1 REPL Mode (Input)

| Key | Action |
|-----|--------|
| `@filename` | Load file (PDF/EPUB) |
| `@@` | Load clipboard contents |
| `@` + Tab | Show file suggestions |
| `:q` | Quit |
| `:h` | Help |

### 7.2 Reading Mode

| Key | Action |
|-----|--------|
| `Space` | Pause/Resume |
| `q` | Quit to REPL |
| `[` / `]` | Decrease/Increase WPM |
| `j` | Jump forward one sentence (always land at sentence beginning) |
| `k` | Jump backward one sentence (always land at sentence beginning) |
| `Tab` (hold) | Peek context (show normal text view) |
| `Shift` (hold) | Tactical throttle (50% speed) |

### 7.3 Future Navigation Enhancements (v1.1+)
**Rapid Navigation Options:**
- **Hold-based advancement**: Holding `j` continuously advances through sentences
- **Count-based jumps**: `5j` jumps forward 5 sentences, `10k` jumps back 10 sentences
- **Time-based skipping**: `30s` skip 30 seconds ahead, `1m` skip 1 minute ahead

**Advanced Navigation Features:**
- **Paragraph jumps**: Jump to next/previous paragraph (larger granularity than sentences)
- **Chapter navigation**: Jump between chapters in EPUB documents
- **Bookmark support**: Mark and return to specific positions
- **Navigation history**: Back/forward through recent positions

**Note:** Current MVP implements single-step sentence navigation (j/k) per Section 7.2.

---

## 8. VISUAL HIERARCHY SUMMARY

| Element | Position | Weight | Research Basis |
| --- | --- | --- | --- |
| **Active Word** | Center | **High** | Foveal Focus. |
| **Anchor Point** | Word Fixation | **Red Pulse** | OVP Fixation. |
| **Progress Line** | Under Word | **1px Dim** | Ambient Pacing. |
| **Marginal Gutter** | Far Right | **Texture** | Spatial Mapping. |
| **Ghost Words** | Left/Right of Center | **15% Opacity** | Context without distraction. |

---

## 9. TERMINAL REQUIREMENTS

### 9.1 Full Feature Support (Recommended)

**Required Terminal:** Konsole (KDE Terminal Emulator) version 22.04+ OR Kitty Terminal

**Required Protocol:** Kitty Graphics Protocol support

**Features Available:**
- Sub-pixel OVP anchoring (pixel-perfect positioning)
- Variable opacity ghost words (true 15% opacity)
- Macro-gutter and micro-bar progress indicators
- Bundled JetBrains Mono font rendering
- 1000+ WPM performance with LRU caching

**Verification:**
```bash
# Check if terminal supports Kitty Graphics Protocol
echo -e "\e[?u\e[c"
# Response should include graphics capability flags
```

### 9.2 Fallback Mode (Universal Compatibility)

**Supported Terminals:** Any terminal with basic ANSI escape sequence support

**Limitations:**
- OVP snaps to nearest character cell (no sub-pixel precision)
- Ghost words use dim attribute (not true opacity)
- Progress bars use Unicode block characters
- Fully functional RSVP reader with reduced visual fidelity

**Performance:** May not achieve 1000+ WPM on slower terminals

### 9.3 Font Configuration

**Default:** JetBrains Mono Regular (bundled, ~300KB)

**Override:** Users can specify custom font path in config:
```toml
[display]
font_path = "/path/to/custom/font.otf"
font_size = 24  # pixels
```

**Requirements:**
- OpenType or TrueType format
- Monospace or proportional (proportional recommended for readability)
- Latin character support minimum (full Unicode planned for future)

---

## 10. DEPENDENCIES

### Core Crates
- `ratatui = "0.30"` - TUI framework (command layer)
- `crossterm = "0.29"` - Terminal I/O
- `rustyline = "17.0"` - REPL implementation
- `pdf-extract = "0.10.0"` - PDF parsing
- `epub = "2.1.5"` - EPUB parsing
- `arboard = "3.6.1"` - Clipboard access
- `thiserror = "2.0.18"` - Error handling

### Graphics Rendering Pipeline
- `ab_glyph = "0.2.32"` - Font rendering and metrics
- `imageproc = "0.25"` - Image manipulation
- `lru = "0.12"` - LRU cache for words
- `base64 = "0.22"` - Kitty protocol encoding

---

## 11. KNOWN LIMITATIONS

### MVP Limitations
- **Terminal Support:** Full features require Konsole/Kitty with Kitty Graphics Protocol
- **Internationalization:** English text only (ab_glyph limitations, no RTL/ligatures)
- **Binary Size:** ~300KB increase from bundled JetBrains Mono font
- **Font Options:** Bundled font only; custom font path config is future work

### Fallback Mode Limitations
- No sub-pixel OVP precision (character-grid only)
- Ghost words use dim attribute instead of true opacity
- Progress bars use Unicode blocks instead of smooth graphics

---

**Document References:**
- Design Doc v2.0: `docs/plans/2026-01-28-TUI Design Doc v2.md`
- Design Review: `docs/plans/2026-01-28-Design Review Summary.md`
- Architecture Doc: `docs/ARCHITECTURE.md`
