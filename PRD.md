
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

Words are horizontally shifted so the **Anchor Letter** remains at a fixed vertical coordinate.

* **Anchor Logic:** * 1 char: 1st letter.
* 2-5 chars: 2nd letter.
* 6-9 chars: 3rd letter.
* 10-13 chars: 4th letter.
* 14+ chars: Cap at 4th position (MVP); Phase 2: Proportional positioning (~33% into word).


* **Salience:** Anchor is colored `#F7768E` (Coral Red) and pulses in luminance at paragraph breaks.

### 3.2 Weighted Delay Algorithm (Non-Linear Timing)

Instead of a static WPM, time-per-word is calculated as:


* **Punctuation:** `.` (3.0x), `,` (1.5x), `\n` (4.0x).
* **Word Length:** Words >10 characters apply configurable delay penalty (default 1.15x, user-adjustable).
* **Chunking:** Common 2-letter pairs (e.g., "in it") are flashed together.

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
* **Peripheral Attenuation:** * **Reading:** 20% Opacity (Subliminal).
* **Paused:** 100% Opacity (Active).


* **Micro-Progress:** A `▁` (U+2581) character beneath the active word shows progress through the current sentence/chapter.

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
| `j` / `k` | Seek backward/forward |
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

