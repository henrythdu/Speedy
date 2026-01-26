
---

# 🚀 SPEEDY: MASTER PRODUCT REQUIREMENTS DOCUMENT

**Project Intent:** A terminal RSVP reader that serves as a pacing and focus tool through foveal anchoring and spatial awareness. Designed to work from any directory, with intuitive file tagging inspired by modern CLI tools.

---

## 1. VISION & SCIENTIFIC FOUNDATION

**Speedy** is built on the principle that speed reading is a matter of **Ocular Efficiency** and **Cognitive Load Management**.

* **OVP (Optimal Viewing Position):** Based on *O'Regan & Lévy-Schoen (1987)*, eye saccades account for ~10% of reading time. RSVP enforces consistent pacing and reduces cognitive friction of eye movement.
* **Modality Effect:** Using auditory cues to provide a temporal metronome for the visual stream.

---

## 2. INPUT MODEL (CLI WORKFLOW)

**Speedy** works from any directory, using intuitive file tagging similar to modern CLI tools like Claude Code.

### 2.1 Launch Patterns

```bash
# From any directory
speedy

# Interactive prompt appears:
speedy> @document.pdf           # Read PDF file
speedy> @@                      # Read clipboard contents
speedy> @chapter.epub           # Read EPUB file
speedy> :q                      # Quit
```

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

* **Background:** `#1A1B26` (Stormy Dark)
* **Text:** `#A9B1D6` (Light Blue) - **7.55:1 contrast** (WCAG AA/AAA compliant)
* **Ghost Context:** Previous/Next words rendered with terminal `dim` attribute (visual salience without accessibility violation).

### 4.2 Navigation & Spatial Awareness (The Gutter)

To solve "Spatial Blindness," a 3-character wide vertical gutter sits on the far right.
* **Progress Indication:** Uses opacity levels and position to map **Information Density** (Phase 2: Consider topographic textures after accessibility validation).
* **Peripheral Attenuation:** * **Reading:** 20% Opacity (Subliminal) - *configurable*.
* **Paused:** 100% Opacity (Active) - *configurable*.

* **Micro-Progress:** A `▁` (U+2581) character beneath the active word shows progress through the current sentence/chapter.

**Gutter Word Display Rules (all configurable via config file):**
* Shows 3 words before current position and 3 words after (total 7 words in gutter) - *word count configurable*
* Words fade from 100% opacity → 20% opacity based on distance from center word:
  - Center word (current): 100% opacity - *configurable*
  - ±1 words from center: 80% opacity - *configurable*
  - ±2 words from center: 60% opacity - *configurable*
  - ±3 words from center: 40% opacity - *configurable*
* Updates continuously during playback
* During navigation (j/k jumps), gutter regenerates around new position immediately

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
├── assets/             # click.wav, config.toml (Embedded via include_bytes!)
├── src/
│   ├── input/          # pdf.rs, epub.rs, clipboard.rs (File parsing)
│   ├── engine/         # timing.rs, ovp.rs (Word positioning, delay logic)
│   ├── ui/             # theme.rs, render.rs (Ratatui/TachyonFX)
│   ├── storage/        # history.rs (Recent files, reading position)
│   ├── app.rs          # State Machine (AppMode Enum)
│   └── main.rs         # Event Loop & REPL
```

### 6.2 Distribution Strategy

* **Single Binary:** Assets embedded in code; self-initializing config in `~/.config/speedy/`.
* **CI/CD:** GitHub Actions for automated binary releases (Mac/Linux/Windows).
* **Install:** `cargo install speedy-rs`.

### 6.3 Technical Implementation Notes

* **Accessibility Compliance:** All functional text meets WCAG AA contrast ratio (≥4.5:1).
* **Performance:** Double-buffered TUI rendering to prevent flicker.
* **Configurable Delays:** Word-length penalty user-adjustable via config file.

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

---

## 8. VISUAL HIERARCHY SUMMARY

| Element | Position | Weight | Research Basis |
| --- | --- | --- | --- |
| **Active Word** | Center | **High** | Foveal Focus. |
| **Anchor Point** | Word Fixation | **Red Pulse** | OVP Fixation. |
| **Progress Line** | Under Word | **1px Dim** | Ambient Pacing. |
| **Marginal Gutter** | Far Right | **Texture** | Spatial Mapping. |

