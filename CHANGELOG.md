# Changelog

All notable changes to Speedy will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - Production-Grade Refactoring Complete

### Phase 6.5: Final Integration and Verification (2026-02-11)

#### Quality Gates ✅
- **Tests**: All 175 tests passing (162 unit + 13 integration)
- **Clippy**: Zero warnings with `-D warnings` flag
- **Formatting**: All code passes `cargo fmt --check`
- **Dead Code**: Eliminated unused imports and methods

#### Code Quality Improvements
- **Removed dead code** from `Token` struct:
  - `text_mut()` method
  - `set_is_sentence_start()` method
  - `push_punctuation()` method
- **Fixed unused imports** in:
  - `src/input/clipboard.rs`
  - `src/input/pdf.rs`
  - `src/ui/autocomplete/mod.rs`
  - `src/rendering/kitty/mod.rs`
  - `src/engine/mod.rs`
- **Marked intentional dead code** with `#[allow(dead_code)]`:
  - `TimingConfig::wpm()` and `set_wpm()` (reserved for future UI)
  - `Theme::dimmed` field and `colors::dimmed()` function

#### Documentation Updates
- Updated `docs/ARCHITECTURE.md` with:
  - Production-grade error handling architecture
  - Input validation layer description
  - Phase 6.5 completion status
  - Recent cleanup history

### Previous Phases Summary

#### Phase 1: Core Implementation
- ✅ PDF/EPUB/Clipboard input support
- ✅ RSVP reading with OVP anchoring
- ✅ WPM adjustment and pause/resume
- ✅ Sentence-aware navigation (j/k keys)

#### Phase 2: Rendering System
- ✅ Kitty Graphics Protocol implementation
- ✅ Word-Level LRU Cache (75MB cap)
- ✅ Sub-pixel OVP positioning
- ✅ Micro progress bar and macro gutter

#### Phase 3: TUI Features
- ✅ File autocomplete with `@` trigger
- ✅ Threaded file discovery
- ✅ Per-directory cache with TTL
- ✅ Midnight theme colors

#### Phase 4-6: Architecture Improvements
- ✅ Error handling with `thiserror` and `anyhow`
- ✅ Input validation layer
- ✅ TerminalBackend trait abstraction
- ✅ Zero unwrap()/expect() in production paths

---

## Known Issues from REFACTORING_ANALYSIS.md

### Addressed ✅
- ✅ **Issue #1**: Error handling infrastructure (`thiserror` + `anyhow`)
- ✅ **Issue #8**: Empty README (content added during phases)
- ✅ **Issue #11**: Configuration validation (WPM clamping)
- ✅ **Issue #14**: Input validation (file path validation)
- ✅ **Issue #15**: Error context preservation (`anyhow::Context`)
- ✅ **Issue #26**: timing.rs organization (documented in ARCHITECTURE)
- ✅ **Issue #27**: Module structure clarity (documented)
- ✅ **Issue #28**: Token encapsulation (fields are private with accessors)

### Outstanding (Future Work)
- 🔲 **Issue #2**: Event loop Command Pattern (architectural improvement)
- 🔲 **Issue #3**: Thread safety poison handling (low risk)
- 🔲 **Issue #4**: Cache eviction iteration limit (safety improvement)
- 🔲 **Issue #5**: Image ID overflow (u32 → u64)
- 🔲 **Issue #6**: O(n) → O(1) token duration optimization
- 🔲 **Issue #7**: UI terminal unit tests (needs TerminalBackend trait completion)
- 🔲 **Issue #9**: Structured logging infrastructure (`tracing`)
- 🔲 **Issue #10**: CI/CD pipeline (GitHub Actions)

---

## Technical Debt Register

| Item | Priority | Status | Notes |
|------|----------|--------|-------|
| Event loop Command Pattern | High | 🔲 Open | Reduces complexity from ~15 |
| Token duration optimization | High | 🔲 Open | O(n) → O(1) per frame |
| TerminalBackend testing | High | 🔲 Open | Enable TUI unit tests |
| Tracing/logging | Medium | 🔲 Open | Production observability |
| CI/CD pipeline | Medium | 🔲 Open | Automated quality gates |
| Image ID overflow | Low | 🔲 Open | u32 → u64, 4.3B renders |
| Cache eviction limit | Low | 🔲 Open | Safety guard |

---

## Performance Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Test Coverage | 80%+ | ~75% | 🟡 Near Target |
| Clippy Warnings | 0 | 0 | ✅ Pass |
| Format Check | Clean | Clean | ✅ Pass |
| Production Panics | 0 | 0 | ✅ Pass |
| Build Time | <30s | ~25s | ✅ Pass |

---

*Last updated: 2026-02-11 by Worker-6.5*
