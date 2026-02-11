# Contributing to Speedy

Thank you for your interest in contributing to Speedy! This document provides guidelines for development setup, building, code style, and submitting pull requests.

## Table of Contents

- [Development Setup](#development-setup)
- [Building](#building)
- [Code Style](#code-style)
- [Testing Guidelines](#testing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Architecture Overview](#architecture-overview)

---

## Development Setup

### Prerequisites

- **Rust 1.70+**: Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Terminal**: Kitty or Konsole 22.04+ (required for Kitty Graphics Protocol)

### Clone and Build

```bash
git clone https://github.com/user/speedy.git
cd speedy
cargo build
```

### Dependencies

Key dependencies are managed automatically by Cargo:
- `ratatui` - TUI framework
- `crossterm` - Terminal I/O
- `ab_glyph` / `imageproc` - Font rendering
- `pdf-extract` / `epub` - File parsing
- `lru` - Word cache

---

## Building

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized performance)
cargo build --release

# Run the application
cargo run

# Run with debug logging
RUST_LOG=debug cargo run
```

---

## Code Style

### Formatting

We use `rustfmt` for consistent code formatting:

```bash
# Format all code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Linting

We use `clippy` with strict warnings:

```bash
# Run linter
cargo clippy -- -D warnings
```

All PRs must pass `cargo clippy` with zero warnings.

### Guidelines

- **No `unwrap()` or `expect()`** in production code paths - use proper error handling
- **Document public APIs** with doc comments (`///`)
- **Follow naming conventions**: snake_case for functions/variables, PascalCase for types
- **Keep functions focused**: Each function should do one thing well

---

## Testing Guidelines

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with verbose output
cargo test -- --nocapture
```

### Test Organization

- **Unit tests**: Located in the same file as the code being tested (inside `#[cfg(test)]` modules)
- **Integration tests**: Located in the `tests/` directory

### Writing Tests

- Write descriptive test names: `test_token_duration_with_punctuation`
- Test edge cases and error conditions
- Aim for high coverage on core logic (`src/reading/`, `src/engine/`)

### Test-Driven Development

We encourage TDD for new features:
1. Write a failing test first
2. Implement minimum code to pass
3. Refactor while keeping tests green

---

## Pull Request Process

### Before Submitting

1. **Create a branch** for your changes
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Run quality gates**
   ```bash
   cargo fmt -- --check  # Formatting
   cargo clippy -- -D warnings  # Linting
   cargo test  # All tests must pass
   ```

3. **Update documentation** if you've changed public APIs or architecture

### PR Requirements

- **All tests pass** (CI will verify)
- **Zero clippy warnings**
- **Code is formatted** (rustfmt)
- **Descriptive commit messages**
- **Link related issues** in PR description

### Review Process

1. Submit PR with clear description of changes
2. Address review feedback promptly
3. Maintain a clean commit history (squash if requested)
4. PR will be merged after approval

---

## Architecture Overview

Speedy uses a **Dual-Engine Architecture**:

1. **Command Layer (Ratatui)**: Standard TUI for command input, progress bars, UI chrome
2. **Reading Layer (Graphics Engine)**: Pixel-perfect rendering via Kitty Graphics Protocol

### Project Structure

```
src/
├── app/           # Application state management
├── engine/        # Shared logic (config)
├── reading/       # Core RSVP logic (tokens, timing, OVP)
├── rendering/     # Graphics backends (Kitty protocol)
├── ui/            # TUI layer (ratatui)
├── input/         # File parsing (PDF, EPUB, clipboard)
└── main.rs        # Entry point
```

For complete architecture details, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Questions?

- Open an issue for bugs or feature requests
- Reference existing issues when submitting PRs
- Be respectful and constructive in all interactions

---

*Thank you for helping make Speedy better!*
