# Speedy

**A terminal-based RSVP (Rapid Serial Visual Presentation) speed reader with pixel-perfect rendering.**

Speedy is a high-performance speed reading application that uses the Kitty Graphics Protocol for pixel-perfect text rendering with Optimal Viewing Position (OVP) anchoring. Built in Rust for maximum performance at 1000+ WPM.

[![CI](https://github.com/user/speedy/actions/workflows/ci.yml/badge.svg)](https://github.com/user/speedy/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

---

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Keybindings](#keybindings)
- [Configuration](#configuration)
- [Architecture](#architecture)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

---

## Features

### Core Reading

- **RSVP Engine**: Words displayed one at a time at your configured WPM
- **OVP Anchoring**: Words are horizontally shifted so the "anchor letter" remains at a fixed position, reducing eye movement
- **Sub-pixel Precision**: Pixel-perfect text positioning using Kitty Graphics Protocol
- **Weighted Timing**: Punctuation and word length affect display duration for natural reading rhythm

### File Support

- **PDF**: Extract and read text from PDF documents
- **EPUB**: Read electronic books in EPUB format
- **Clipboard**: Paste text directly from your clipboard

### Navigation

- **Sentence-aware jumps**: Navigate forward/backward by sentence (always lands at sentence beginning)
- **WPM adjustment**: Increase or decrease reading speed on the fly
- **Pause/Resume**: Stop and continue reading at any point

### Visual Design

- **Midnight Theme**: Dark theme designed to minimize eye strain with WCAG AA contrast ratios
- **Progress Indicators**:
  - Micro-bar: Horizontal progress through current sentence
  - Macro-gutter: Vertical progress through entire document
- **Mode-aware opacity**: Progress dims during active reading, brightens when paused

### Performance

- **Word-Level LRU Cache**: Eliminates redundant rasterization for consistent 1000+ WPM
- **75MB memory cap**: Automatic cache eviction to prevent memory bloat
- **~70% cache hit rate**: Typical English text achieves high cache efficiency

---

## Requirements

### Terminal (Required)

Speedy requires a terminal with **Kitty Graphics Protocol** support:

| Terminal | Version | Status |
|----------|---------|--------|
| **Kitty** | Any | ✅ Recommended |
| **Konsole** | 22.04+ | ✅ Supported |

**Why?** Speedy uses pixel-perfect graphics rendering for sub-pixel OVP anchoring. Standard terminal text rendering cannot achieve this precision.

### System

- **Rust**: 1.70+ (for building from source)
- **OS**: Linux, macOS (Windows support planned)

### Minimum Terminal Size

- 80 columns × 24 rows

---

## Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/user/speedy.git
cd speedy

# Build and install
cargo install --path .
```

### Using Cargo

```bash
cargo install speedy
```

---

## Quick Start

1. **Launch Speedy:**
   ```bash
   speedy
   ```

2. **Load a file** (in the command deck at the bottom):
   ```
   @document.pdf
   ```

3. **Start reading!** The app automatically begins at 300 WPM.

4. **Control your reading:**
   - Press `Space` to pause/resume
   - Press `]` to increase WPM, `[` to decrease
   - Press `j` to jump forward one sentence
   - Press `k` to jump backward one sentence

---

## Usage

### Command Deck

Speedy uses a command deck at the bottom of the terminal for input. Type commands directly without a prompt.

#### Loading Files

| Command | Description |
|---------|-------------|
| `@path/to/file.pdf` | Load a PDF file |
| `@path/to/book.epub` | Load an EPUB file |
| `@@` | Load text from clipboard |

#### File Autocomplete

When typing `@`, a popup appears with file suggestions:

- **↑/↓**: Navigate file list
- **Enter/Tab**: Select file
- **Esc**: Cancel
- **Ctrl+R**: Refresh file cache

#### Commands

| Command | Description |
|---------|-------------|
| `:q` or `:quit` | Exit application |
| `:h` or `:help` | Show help |

---

## Keybindings

### Reading Mode

| Key | Action |
|-----|--------|
| `Space` | Pause / Resume |
| `q` | Return to command mode |
| `]` | Increase WPM by 50 |
| `[` | Decrease WPM by 50 |
| `j` | Jump forward one sentence |
| `k` | Jump backward one sentence |

### Command Mode

| Key | Action |
|-----|--------|
| `@` | Start file path (triggers autocomplete) |
| `:` | Start command |
| `Enter` | Execute command |
| `Esc` | Cancel |

### Autocomplete Popup

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate file list |
| `Enter` / `Tab` | Select highlighted file |
| `Esc` | Close popup |
| `Ctrl+R` | Refresh file cache |

---

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level | `info` |

Example:
```bash
RUST_LOG=debug speedy
```

### Config Directory

Speedy stores configuration in:
```
~/.config/speedy/
```

### Default Settings

| Setting | Value | Description |
|---------|-------|-------------|
| Default WPM | 300 | Initial reading speed |
| WPM Range | 50-1000 | Minimum and maximum WPM |
| Cache Size | 1000 words | LRU cache capacity |
| Memory Cap | 75 MB | Maximum cache memory |

---

## Architecture

Speedy uses a **Dual-Engine Architecture**:

### 1. Command Layer (Ratatui)
- Standard character-grid TUI
- Command input, progress bars, UI chrome

### 2. Reading Layer (Graphics Engine)
- Pixel-perfect rendering via Kitty Graphics Protocol
- Sub-pixel OVP anchoring
- True opacity for visual effects

### Project Structure

```
src/
├── app/           # Application state management
├── reading/       # Core RSVP logic (tokens, timing, OVP)
├── rendering/     # Graphics backends (Kitty protocol)
├── ui/            # TUI layer (ratatui)
├── input/         # File parsing (PDF, EPUB, clipboard)
└── main.rs        # Entry point
```

For full architecture details, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Troubleshooting

### "Terminal does not support Kitty Graphics Protocol"

**Solution**: Use Kitty or Konsole (22.04+). Other terminals are not supported.

### Text appears garbled or misaligned

1. Ensure you're using a supported terminal
2. Try resizing the terminal and restarting
3. Check that your terminal font size is reasonable (12-16pt recommended)

### Performance issues at high WPM

1. The word cache needs time to warm up - performance improves after the first pass
2. Try reducing WPM temporarily to let the cache populate
3. Ensure your system has available memory (75MB cache cap)

### File not loading

1. Check the file path is correct
2. Ensure the file is a valid PDF or EPUB
3. Some PDFs with complex layouts may not extract text cleanly

---

## Contributing

Contributions are welcome! Please read the architecture documentation before submitting PRs.

### Development Setup

```bash
# Clone and build
git clone https://github.com/user/speedy.git
cd speedy
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

---

## License

This project is dual-licensed under:

- **MIT License**
- **Apache License 2.0**

### Bundled Font

This project bundles **JetBrains Mono**, which is licensed under the SIL Open Font License 1.1.

See [licenses/JetBrainsMono-LICENSE.txt](licenses/JetBrainsMono-LICENSE.txt) for the full license text.

Copyright 2020 The JetBrains Mono Project Authors  
https://github.com/JetBrains/JetBrainsMono

---

## Acknowledgments

Speedy is built on research in ocular efficiency and cognitive load management:

- **OVP (Optimal Viewing Position)**: Based on O'Regan & Lévy-Schoen (1987) - eye saccades account for ~10% of reading time
- **RSVP**: Reduces cognitive friction of eye movement through consistent pacing

---

*Built with ❤️ in Rust*
