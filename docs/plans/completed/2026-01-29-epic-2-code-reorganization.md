# Epic 2: Codebase Reorganization & Architecture Refinement

**Status:** Draft - Awaiting Validation  
**Created:** 2026-01-29  
**Version:** 1.0 Draft

---

## 1. Overview

This epic addresses codebase organization issues identified during Epic 1 implementation. While Epic 1 established the core TUI foundation with dual-engine architecture, the codebase has accumulated organizational debt: large files with mixed responsibilities, inconsistent module structure, and root-level files that should be namespaced.

### 1.1 Scope
- **Restructure src/ directory with domain-based folder organization**
- **Split large files (>300 lines) into focused modules**
- **Move root-level files into appropriate modules**
- **Standardize module boundaries and public APIs**
- **Update ARCHITECTURE.md to reflect new structure**
- **Maintain 100% test compatibility throughout reorganization**

### 1.2 Why This Epic Now?
Epic 1 is functionally complete (all tests passing). Before Epic 2 features (ghost words, caching, progress bars) are implemented, we need clean module boundaries. Adding new features to the current disorganized structure will compound the technical debt and slow development velocity.

### 1.3 Guiding Principles
1. **Domain-driven organization** - Group by feature/concern, not layer
2. **Single responsibility** - Each module has one clear purpose
3. **Shallow imports** - Prefer `use crate::feature::Type` over deep nesting
4. **Preserve public APIs** - Reorganization is internal; external API stays stable
5. **Incremental validation** - Each bead must pass all tests before proceeding

---

## 2. Background & Context

### 2.1 Current State Analysis

**File size distribution:**
- `src/app/app.rs` - 607 lines (MIXED: App struct, AppEvent, RenderState, event handlers)
- `src/engine/state.rs` - 489 lines (MIXED: ReadingState, navigation, timing)
- `src/engine/timing.rs` - 370 lines (MIXED: Token, punctuation, timing, tokenization)
- `src/engine/cell_renderer.rs` - 321 lines (ACCEPTABLE: focused renderer impl)
- `src/engine/viewport.rs` - 292 lines (ACCEPTABLE: coordinate management)
- `src/ui/reader_component.rs` - 167 lines (ACCEPTABLE: UI component)

**Structural issues:**
- `src/terminal.rs` at root level - should be in `src/ui/`
- `src/ui/` mixes concerns: terminal, reader, render, theme, command
- `src/engine/` has flat structure despite having many components
- Tests are embedded in source files, making files larger

### 2.2 Target Architecture

**Domain-based folder structure:**
```
src/
├── lib.rs                    # Public exports only
├── main.rs                   # Entry point
├── app/                      # Application state & coordination
│   ├── mod.rs
│   ├── state.rs              # App struct, mode management
│   ├── event.rs              # AppEvent enum, event dispatch
│   ├── render_state.rs       # RenderState, render coordination
│   └── app.rs                # Main App impl (reduced size)
├── reading/                  # Reading engine (pure logic)
│   ├── mod.rs
│   ├── state.rs              # ReadingState (simplified)
│   ├── token.rs              # Token struct, tokenization
│   ├── timing.rs             # WPM calculations, delay logic
│   ├── navigation.rs         # Jump forward/backward by sentence
│   └── ovp.rs                # OVP calculation (unchanged)
├── rendering/                # All rendering backends
│   ├── mod.rs
│   ├── trait.rs              # RsvpRenderer trait
│   ├── cell.rs               # CellRenderer (TUI fallback)
│   ├── viewport.rs           # Viewport coordinate management
│   ├── font.rs               # Font loading & metrics
│   └── capability.rs         # Terminal capability detection
├── ui/                       # UI layer (Ratatui-based)
│   ├── mod.rs
│   ├── terminal.rs           # TuiManager, terminal handling
│   ├── theme.rs              # Colors, styling
│   ├── reader/
│   │   ├── mod.rs
│   │   ├── component.rs      # ReaderComponent
│   │   └── view.rs           # render_word_display, etc.
│   └── command.rs            # Command parsing (future use)
├── input/                    # File input (unchanged)
│   ├── mod.rs
│   ├── pdf.rs
│   ├── epub.rs
│   └── clipboard.rs
├── audio/                    # Stub for future audio epic
│   └── mod.rs
└── storage/                  # Stub for future storage epic
    └── mod.rs
```

### 2.3 Key Technical Decisions
1. **Split by domain, not by layer** - `reading/` contains all reading-related logic regardless of layer
2. **Separate tests from implementation** - Move tests to `tests/` directory or inline test modules
3. **Preserve exports** - `src/lib.rs` public API remains unchanged
4. **Incremental migration** - One module at a time, validate after each

---

## 3. Tasks

### Task 1: Split src/app/app.rs into focused modules
**Priority:** P0 - Core application structure  
**Estimated Effort:** 2-3 days

**Description:**
`src/app/app.rs` (607 lines) mixes App struct, AppEvent enum, RenderState struct, and event handlers. Split into separate files with clear responsibilities.

**Technical Details:**
1. **Extract AppEvent** (`src/app/event.rs`)
   - Move `AppEvent` enum and its logic
   - Keep event variants: LoadFile, LoadClipboard, Quit, Help, Warning, InvalidCommand

2. **Extract RenderState** (`src/app/render_state.rs`)
   - Move `RenderState` struct and factory methods
   - Keep fields: mode, current_word, tokens, current_index, context_left, context_right, progress

3. **Simplify src/app/app.rs**
   - Keep only `App` struct and core methods
   - Import from new modules
   - Target size: <200 lines

**Acceptance Criteria:**
- [ ] `src/app/app.rs` <200 lines after split
- [ ] `src/app/event.rs` contains AppEvent
- [ ] `src/app/render_state.rs` contains RenderState
- [ ] All tests still pass (176 tests)
- [ ] No breaking changes to public API
- [ ] ARCHITECTURE.md updated

**Dependencies:** None

---

### Task 2: Split src/engine/ into reading/ and rendering/
**Priority:** P0 - Core domain separation  
**Estimated Effort:** 3-4 days

**Description:**
`src/engine/` mixes reading state management with rendering concerns. Split into two domains: `reading/` (pure reading logic) and `rendering/` (all backends).

**Technical Details:**
1. **Create src/reading/ module**
   - Move `src/engine/state.rs` → `src/reading/state.rs` (simplified)
   - Move `src/engine/timing.rs` → split into:
     - `src/reading/token.rs` (Token struct, tokenization)
     - `src/reading/timing.rs` (WPM, delay calculations)
   - Move `src/engine/ovp.rs` → `src/reading/ovp.rs`
   - Create `src/reading/navigation.rs` (extract from state.rs)

2. **Create src/rendering/ module**
   - Move `src/engine/renderer.rs` → `src/rendering/trait.rs`
   - Move `src/engine/cell_renderer.rs` → `src/rendering/cell.rs`
   - Move `src/engine/viewport.rs` → `src/rendering/viewport.rs`
   - Move `src/engine/font.rs` → `src/rendering/font.rs`
   - Move `src/engine/capability.rs` → `src/rendering/capability.rs`

3. **Handle config.rs and error.rs**
   - Keep in `src/engine/` OR distribute to relevant modules
   - Evaluate: config.rs has ThemeConfig, GutterConfig, AudioConfig, TactileConfig
   - Decision: Keep error.rs in engine/, move configs to respective domains

**Acceptance Criteria:**
- [ ] `src/reading/` module created with all reading logic
- [ ] `src/rendering/` module created with all rendering logic
- [ ] No files >300 lines in new modules (except tests)
- [ ] All tests still pass
- [ ] Public API preserved through re-exports
- [ ] ARCHITECTURE.md updated

**Dependencies:** Task 1

---

### Task 3: Reorganize src/ui/ into feature folders
**Priority:** P1 - UI layer cleanup  
**Estimated Effort:** 2-3 days

**Description:**
`src/ui/` mixes terminal management, reader components, theme, and command parsing. Organize into feature folders.

**Technical Details:**
1. **Move root-level terminal.rs**
   - `src/terminal.rs` → `src/ui/terminal_guard.rs` (or merge into terminal.rs)

2. **Create src/ui/reader/ subdirectory**
   - Move `src/ui/reader_component.rs` → `src/ui/reader/component.rs`
   - Move `src/ui/render.rs` → `src/ui/reader/view.rs`
   - Keep `src/ui/reader/mod.rs` for exports

3. **Clean up src/ui/mod.rs exports**
   - Remove unused exports: `command_to_app_event`, `parse_command`, `Command`
   - Keep only actively used: `TuiManager`, `ReaderComponent` (via reader/)

4. **Evaluate command.rs**
   - Currently unused (command parsing)
   - Decision: Keep as stub for future command deck implementation

**Acceptance Criteria:**
- [ ] `src/terminal.rs` moved into `src/ui/`
- [ ] `src/ui/reader/` subdirectory created
- [ ] No unused exports in `src/ui/mod.rs`
- [ ] All tests still pass
- [ ] ARCHITECTURE.md updated

**Dependencies:** Task 2

---

### Task 4: Update public API and documentation
**Priority:** P1 - API stability  
**Estimated Effort:** 1-2 days

**Description:**
Ensure `src/lib.rs` exports remain stable and update all documentation.

**Technical Details:**
1. **Update src/lib.rs**
   - Keep existing module structure OR update to new domains
   - If changing: provide deprecation warnings for old paths
   - Decision: Keep lib.rs simple, re-export from new locations

2. **Update ARCHITECTURE.md**
   - Rewrite Section 1 (Project Structure) with new folder layout
   - Update Section 2 (Core Structs) with new file locations
   - Update Section 3 (Public Methods) with new paths
   - Keep all other sections relevant

3. **Update PRD if needed**
   - Check if any PRD sections reference old file paths
   - Update to new structure

**Acceptance Criteria:**
- [ ] `src/lib.rs` exports stable and working
- [ ] ARCHITECTURE.md fully updated
- [ ] PRD references updated (if any)
- [ ] All tests still pass

**Dependencies:** Task 3

---

## 4. Human Testing Bead

### Bead 5: Code Review & Architecture Validation
**Type:** Manual Review  
**Estimated Time:** 1-2 hours

**Review Checklist:**
1. **Module Boundaries:**
   - [ ] Each module has single, clear responsibility
   - [ ] No circular dependencies between modules
   - [ ] Public APIs are minimal and intentional

2. **File Sizes:**
   - [ ] No source files >300 lines (except test files)
   - [ ] Tests separated from implementation where possible

3. **Naming Conventions:**
   - [ ] Module names consistent with content
   - [ ] File paths reflect domain structure
   - [ ] No confusing naming (e.g., reader.rs vs reader_component.rs issue resolved)

4. **Import Paths:**
   - [ ] Internal imports use crate-relative paths
   - [ ] No deep nesting (max 3 levels: crate::domain::module)
   - [ ] Public exports documented

**Deliverable:** Architecture validation report in `docs/testing/epic2-code-review.md`

---

## 5. Dependencies

### 5.1 No New Dependencies
This epic is pure reorganization - no new crates needed.

### 5.2 Development Tools
- `cargo test` - Verify tests after each bead
- `cargo check` - Verify compilation
- `cargo clippy` - Check for lint issues

---

## 6. Success Criteria

### 6.1 Must Have (MVP)
- [ ] All files organized into domain-based folders
- [ ] No source files >300 lines (except tests)
- [ ] Root-level `src/terminal.rs` moved into module
- [ ] All 176 tests passing
- [ ] No compilation errors or warnings
- [ ] ARCHITECTURE.md updated to reflect new structure
- [ ] Public API remains stable

### 6.2 Should Have
- [ ] Clear module boundaries with single responsibilities
- [ ] Tests separated from implementation (where practical)
- [ ] Import paths simplified and consistent
- [ ] Dead code eliminated

### 6.3 Nice to Have
- [ ] Module-level documentation (README.md in each folder)
- [ ] Architecture Decision Records (ADRs) for major changes
- [ ] Pre-commit hooks for organization validation

---

## 7. Risk Assessment

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| **Broken imports during migration** | High | High | Incremental migration, validate after each bead |
| **Lost git history/blame** | Low | Medium | Use `git mv` for file moves, preserve history |
| **Public API breakage** | High | Medium | Careful re-exports in lib.rs, comprehensive testing |
| **Developer confusion** | Medium | Low | Update ARCHITECTURE.md immediately, clear naming |
| **Merge conflicts with parallel work** | Medium | Medium | Coordinate with team, complete Epic 1 first |

---

## 8. Timeline

**Total Duration:** 1-1.5 weeks  
**Parallel Work:** None (sequential dependencies)  
**Buffer:** +2 days for unexpected issues

---

## 9. Next Epic Direction (Brief)

After code reorganization completes, Epic 3 will focus on:
1. **Ghost Word Rendering** - Implement peek-ahead and look-behind words
2. **Word-Level LRU Cache** - Performance optimization for 1000+ WPM
3. **Progress Indicators** - Macro-gutter and micro-bar implementation

Clean module boundaries from this epic will make Epic 3 implementation straightforward.

---

## 10. Notes

### 10.1 Why Not Do This During Epic 1?
Epic 1 was focused on functionality and architecture validation. Attempting reorganization simultaneously would have:
- Slowed down feature delivery
- Created confusion about what was "old" vs "new" code
- Made debugging harder (was the bug in old or new structure?)

### 10.2 Rust Module System
Rust's module system supports both:
- **File modules:** `src/foo.rs` defines `crate::foo`
- **Folder modules:** `src/foo/mod.rs` defines `crate::foo`
- **Submodules:** `src/foo/bar.rs` defines `crate::foo::bar`

We'll use folder modules for domains with multiple files, file modules for simple cases.

### 10.3 Testing Strategy
- Unit tests move with their implementations
- Integration tests in `tests/` directory remain unchanged
- Each bead includes: move code → fix imports → run tests → verify

### 10.4 Validation Results

**Consensus Building (3 models):**
- Gemini (for, 9/10): Strong support for domain-driven organization
- GPT-5.1-codex (against, 6/10): Concerns about timing and scope
- Claude (neutral, 8/10): Balanced synthesis of both perspectives

**Key Agreements:**
- Domain-driven organization > layered architecture
- Rust compiler + 176 tests provide safety net
- Timing is correct (post-Epic 1, pre-Epic 3)
- Use `pub use` re-exports for API stability
- Cohesion matters more than line count

**Synthesized Modifications:**
1. Rename: `reading/` → `rsvp/`, `rendering/` → `display/` (clearer semantics)
2. Raise threshold: 400-500 lines (not 300) for Rust
3. Timeline: 2-2.5 weeks realistic for full scope
4. Approach: Phased - high-impact low-risk moves first
5. Validate effectiveness during Epic 3

**Challenge Assumptions:**
- Module boundaries must be explicitly documented before moves
- Dependency flow must be defined (rsvp/ independent, display/ service layer, app/ orchestration)
- Phased approach allows early validation and course correction
- Risk of NOT reorganizing exceeds risk of careful reorganization

**Deep Analysis Expert Recommendations:**
1. Document module boundaries and dependency rules in ARCHITECTURE.md FIRST
2. Plan move sequence by dependency (leaves → core services → entrypoints)
3. Use `#[deprecated]` on pub use re-exports for guided migration
4. Prioritize Single Responsibility Principle over line count

---

## 11. Approval

**Plan Status:** ✅ **VALIDATED AND READY FOR IMPLEMENTATION**

This plan has been validated through:
- Consensus building (3 models, confidence 8/10)
- Challenge of assumptions (blind spots identified)
- Deep analysis (expert recommendations)

**Validation Confidence: 8/10**

**Next Step:** Create beads using `bd create` for each task

---

*This document follows the AGENTS.md epic planning workflow: Consensus → Challenge → Synthesis → Plan Documentation*
