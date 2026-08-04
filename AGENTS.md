# Agent Instructions

This project uses **cells** for code organization and `cells health` as the partition gate. Work one cell at a time instead of drowning in the whole codebase.

## Quick Reference

```bash
cells list              # Partition overview: cells, sizes, fan-in/out, orphans
cells show <cell>       # A cell's membrane + crossings + size
cells payload <cell>    # Full context (membrane + owned files + neighbors' membranes)
cells impact <cell>     # Blast radius before editing (who transitively depends on this)
cells health            # The gate: crossings, structure (acyclic/layered), size, grammars
cells crossings         # Drill into undeclared/stale crossings
cells structure         # Layers + Stable-Dependencies-Principle check
cells owns <file>       # Which cell owns this file? (reverse lookup)
```

**Rule:** `cells health` must stay green. If a change makes it red, fix the membrane (`.cell.toml` requires/provides, reassign files, or remove dead imports) and re-run until green.

---

## Epic Workflow

### 1. Epic Planning

When planning epics for the project:

**Current Epic:**

- Create detailed plan with technical specifications
- Include implementation details, dependencies, and acceptance criteria
- Define clear boundaries and scope

**Next Epic:**

- Create brief plan only (directional, no technical details)
- High-level overview of intended work
- Focus on "what" and "why," not "how"
- 2-3 bullet points maximum

**Principle:** Only detail plan what you're about to work on. Keep the pipeline lean.

---

### 2. Starting a New Epic

Before detailed planning begins:

1. **Review Previous Epic:**
   - Check what was delivered
   - Identify dependencies or blockers
   - Note lessons learned

2. **Align with PRD:**
   - Read current @PRD.md
   - Ensure epic supports product vision
   - Identify any PRD gaps or updates needed
   - Update PRD if epic introduces new requirements

**Principle:** Never plan in isolation. Always connect to what came before and product intent.

---

### 3. Plan Validation Workflow

**MANDATORY for ALL Epics:** Before starting any development work on any epic, you MUST complete all three validation steps below in order.

After completing detailed plan for current epic, follow this sequence:

#### Step 1: Consensus Building

```bash
pal_consensus
```

- Present the epic plan to multiple AI models with different perspectives
- Use stances: `for`, `against`, `neutral` to get balanced feedback
- Goal: Reach agreement on plan viability and completeness
- Output: Refined plan with consensus on key decisions

#### Step 2: Challenge Assumptions

```bash
pal_challenge
```

- Explicitly challenge the plan and underlying assumptions
- Question every constraint, requirement, and approach
- Force critical thinking to avoid reflexive agreement
- Goal: Identify blind spots, risks, and alternatives

#### Step 3: Deep Analysis & Recommendation

```bash
pal_thinkdeep
```

- Use continuation_id from consensus work
- Provide perspective on both consensus and challenge outcomes
- Synthesize insights into actionable recommendations
- Goal: Final, validated plan with clear action items

**Workflow Order:** Consensus → Challenge → ThinkDeep (no shortcuts, all three required)

---

### 4. Task Planning

After user approves the epic plan:

1. Break down the epic into individual tasks
2. Each task must:
   - Have clear acceptance criteria
   - Be independently testable
   - Fit within a focused work session
   - Stay within one cell (or explicitly plan the crossing)
3. Use `cells list` to orient; keep the partition's layering (see `cells structure`)

**Principle:** Small, focused tasks are easier to complete and review.

---

### 5. Code Explanation Requirements

When coding agents propose code:

**Mandatory Context:**

- Why this function/module is needed
- Where it fits into the big picture
- How it connects to PRD requirements
- What problem it solves

**Example Explanation Format:**

```
Purpose: [What this code accomplishes]
Big Picture: [How it enables the epic/feature]
PRD Reference: [Section number from PRD.md]
Connections: [What modules depend on this / what this depends on]
```

**Principle:** Never implement without context. Code must tell a story.

---

### 6. Test-Driven Development (TDD)

**Mandatory Process:** All feature implementation MUST follow TDD:

1. **Write Test First:**
   - Before writing any implementation code, write a failing test
   - Test should specify expected behavior clearly
   - Use descriptive test names (e.g., `test_word_delay_calculation_at_300_wpm`)

2. **Run Test (Verify Failure):**
   - Execute test and confirm it fails as expected
   - This validates test actually tests the intended behavior

3. **Implement Minimum Code:**
   - Write simplest code to make test pass
   - No extra features or optimizations
   - Focus on meeting test requirements only

4. **Run Test (Verify Success):**
   - Execute test and confirm it passes
   - If it doesn't pass, fix code (not test)

5. **Refactor (If Needed):**
   - Clean up code while keeping tests green
   - Remove duplication, improve clarity
   - Re-run tests to ensure nothing broke

**Testing Tools for This Project:**

- Use `cargo test` for unit tests in `engine/` (pure logic, no UI)
- Integration tests in `tests/` directory for end-to-end scenarios
- Mock terminal I/O for testing `ui/` components

**Principle:** Never write implementation code without a failing test. Tests define requirements.

---

### 7. Architecture Documentation

**MANDATORY:** Maintain `docs/ARCHITECTURE.md` as source of truth for codebase structure.

**Purpose:** Prevent duplicate work and confusion by documenting what actually exists.

**When to Update Architecture Doc:**

1. **Adding new public methods** to existing structs
2. **Creating new modules** or files  
3. **Changing architecture** patterns
4. **Adding significant dependencies**
5. **Completing major features** (update implementation status)

**When NOT to Update:**

1. **Test-only changes**
2. **Private method additions**
3. **Refactors without API changes**
4. **Bug fixes** (unless architecture impacted)

**Documentation Standards:**

- Use `file_path:line_number` format for references
- Keep descriptions brief and factual
- Only document WHAT EXISTS, not planned code
- Include purpose for each struct/module

**Verification Method:** Cross-reference with actual codebase using `serena_search_for_pattern`

### 8. Pre-Commit Workflow with Architecture Updates

After an epic's work is complete, before committing code:

```bash
pal_precommit
```

- Validates git changes systematically
- Reviews diffs and impacts
- Checks for security issues, missing tests, incomplete features
- Provides structured investigation with expert analysis
- Must pass all validation before git commit

**Mandatory Steps:**

1. Run quality gates (tests, linters, builds)
2. Run `pal_precommit` on all changes
3. Address any issues found
4. **UPDATE ARCHITECTURE DOC** if architectural changes were made
5. Commit only after precommit passes AND architecture doc is updated

**Architecture Update Process:**

1. **Read current ARCHITECTURE.md** to understand baseline
2. **Review git changes** to identify architectural impacts
3. **Update relevant sections** with new methods/modules
4. **Verify accuracy** by cross-referencing with codebase
5. **Update "Last Updated"** date at top of document

**Principle:** Quality gates AND accurate documentation are non-negotiable. Never bypass precommit validation or skip architecture updates.

---

### 9. Handling Syntax Errors & Library Changes

When encountering syntax errors or unexpected behavior with external libraries:

1. **Use Context7 for Latest Documentation:**

   ```bash
   pal_apilookup
   ```

   - Fetches current API documentation from authoritative sources (official docs, GitHub)
   - Use when: syntax errors, breaking changes, deprecations, migration guides needed
   - Critical for: `ratatui`, `crossterm`, `rodio`, or any dependency newer than your knowledge cutoff
   - Always verify before attempting fixes - may be recent API changes

2. **Common Scenarios Requiring API Lookup:**
   - "Cannot find trait `X` in crate `Y`" - API may have changed
   - "Method `Z` doesn't exist" - Version mismatch or breaking change
   - "Error: unexpected type parameter" - Generic signatures updated
   - Import resolution failures - Module structure reorganized

**Principle:** Never guess at API fixes. Always fetch current docs first.

---

### 10. Serena Tools for Project Management

Serena provides powerful code analysis and project management tools. Use these for:

**Codebase Navigation & Understanding:**

- `serena_get_symbols_overview` - Quick overview of file structure
- `serena_find_symbol` - Locate specific functions/classes
- `serena_find_referencing_symbols` - Find where symbols are used
- `serena_search_for_pattern` - Search for code patterns across codebase
- `serena_list_dir` - Explore project structure

**Code Modification:**

- `serena_replace_content` - Replace code sections (regex supported)
- `serena_replace_symbol_body` - Replace entire function/class bodies
- `serena_insert_before_symbol` / `serena_insert_after_symbol` - Add code near symbols
- `serena_rename_symbol` - Rename symbols across entire codebase

**Project Intelligence:**

- `serena_write_memory` - Store project knowledge for future sessions
- `serena_read_memory` - Retrieve stored project context
- `serena_list_memories` - View available project memories

**Best Practices:**

- Use symbol-based operations (`serena_find_symbol`, `serena_replace_symbol_body`) when working with specific functions/classes
- Use pattern search (`serena_search_for_pattern`) when structure is unknown
- Write memories for critical architectural decisions and design patterns
- Always read memory before starting new work to understand project context

---

### 11. Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds, `cells health`
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:

   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```

5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**

- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
