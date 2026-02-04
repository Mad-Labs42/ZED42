# ZED42 Development Rules and Enforcement

**MANDATORY COMPLIANCE DOCUMENT**

All AI assistants, developers, and contributors MUST follow these rules without exception.

---

## Rule 0: Read Before Every Task

**Before beginning ANY development work:**

1. ✅ Read `STYLE_GUIDE.md` in full
2. ✅ Check current file sizes: `find crates -name "*.rs" -exec wc -l {} \;`
3. ✅ Review architecture constraints in `ARCHITECTURE.txt`
4. ✅ Verify you understand the task's scope

**Failure to read these documents before coding is a critical violation.**

---

## Rule 1: File Size Enforcement

### Absolute Limits (NEVER EXCEED)

```
Module files (lib.rs, mod.rs):     300 LOC - HARD LIMIT
Implementation files:              400 LOC - HARD LIMIT
Test files:                        500 LOC - HARD LIMIT
Configuration files:               200 LOC - HARD LIMIT
```

### When Approaching Limits (>75% of max):

**REQUIRED ACTION**: Stop coding and refactor BEFORE adding more code.

### Refactoring Process:

1. **Analyze responsibilities**
   ```bash
   # Count logical sections in file
   rg "^(pub )?fn " filename.rs | wc -l
   ```

2. **Create module directory**
   ```bash
   mkdir -p crates/module_name/src/submodule
   ```

3. **Split by responsibility**
   - Extract types → `types.rs`
   - Extract core logic → `core.rs` or specific feature files
   - Keep tests separate → `tests.rs`

4. **Update mod.rs with public API**
   ```rust
   // mod.rs - PUBLIC API ONLY
   mod types;
   mod core;

   pub use types::*;
   pub use core::MainStruct;
   ```

### Examples of Violations:

❌ `session.rs` - 570 LOC (VIOLATION - exceeds 400 LOC limit)
❌ `knowledge_graph.rs` - 588 LOC (VIOLATION - exceeds 400 LOC limit)
❌ `working.rs` - 366 LOC (WARNING - approaching limit)

**These MUST be refactored before continuing development.**

---

## Rule 2: Function Size Enforcement

### Limits

```
Public functions:    50 LOC max
Private functions:   30 LOC max
Test functions:      60 LOC max
Constructors:        40 LOC max
```

### Enforcement Strategy

**Before writing a function:**
1. Outline the logic in comments
2. If outline >20 lines, plan to split
3. Extract helpers proactively

**If function exceeds limit:**
1. Extract private helpers
2. Use builder pattern for complex construction
3. Simplify conditional logic
4. Question necessity of complexity

---

## Rule 3: No God Objects

### Detection Criteria

A struct is a God Object if ANY of:
- Has >15 public methods
- Has >10 fields
- Total LOC (struct + all impls) >500
- Depends on >5 other modules
- Has >3 responsibilities

### Enforcement

**When detected:**
1. **STOP immediately** - Do not add more code
2. **Identify responsibilities** - List what the object does
3. **Extract by responsibility** - Create focused structs
4. **Use composition** - Delegate to smaller objects

### Example Violations:

Current `SessionMemory` approaches God Object:
- 15+ methods (insert, get, search, get_recent, prune, stats, checkpoint, etc.)
- Multiple responsibilities (DB, search, stats, schema)

**Required Fix**: Split into:
```rust
SessionMemory (coordinator - 5 methods)
├── SessionDatabase (CRUD - 6 methods)
├── SessionSearch (FTS5 - 4 methods)
└── SessionStats (metrics - 3 methods)
```

---

## Rule 4: Module Organization

### Directory Structure (MANDATORY for >2 files)

```
module_name/
├── mod.rs          (30-50 LOC) - Public API only
├── types.rs        (up to 200 LOC) - Data structures
├── core.rs         (up to 300 LOC) - Main implementation
├── helpers.rs      (up to 200 LOC) - Private utilities
└── tests.rs        (up to 500 LOC) - All tests
```

### When to Use Subdirectories

**Create subdirectory when:**
- Module has >3 implementation files
- Clear sub-domains exist
- Exceeding file count limits

**Example:**
```
memory/
├── mod.rs
├── working/
│   ├── mod.rs
│   ├── cache.rs
│   ├── eviction.rs
│   └── tests.rs
├── session/
│   ├── mod.rs
│   ├── database.rs
│   ├── search.rs
│   └── tests.rs
└── knowledge_graph/
    ├── mod.rs
    ├── nodes.rs
    ├── edges.rs
    ├── search.rs
    └── tests.rs
```

---

## Rule 5: Documentation Requirements

### Mandatory Documentation

**ALL public items require:**
```rust
/// One-line summary
///
/// Detailed description
///
/// # Arguments
/// - `param` - Description
///
/// # Returns
/// Description
///
/// # Errors
/// When errors occur
///
/// # Examples
/// ```
/// example_code();
/// ```
pub fn function() -> Result<()> { }
```

**Private items require:**
```rust
/// Brief description of purpose
fn helper() { }
```

### Documentation Violations

❌ Missing docs on public functions
❌ Generic "does a thing" descriptions
❌ Undocumented error cases
❌ No examples for complex APIs

---

## Rule 6: Error Handling

### Required Practices

✅ Use `Result<T, E>` for all fallible operations
✅ Use `anyhow::Result` in application code
✅ Add `.context("description")` to all error chains
✅ Use `thiserror` for library error types

### Forbidden Practices

❌ `unwrap()` in production code
❌ `expect("")` without detailed message
❌ Silently ignoring errors with `let _ =`
❌ Generic error messages

### Example:

✅ **CORRECT**:
```rust
let file = File::open(path)
    .context("Failed to open config file")?;
```

❌ **WRONG**:
```rust
let file = File::open(path).unwrap(); // FORBIDDEN
```

---

## Rule 7: Testing Requirements

### Coverage Standards

- Public functions: 100% coverage
- Private functions: >80% coverage
- Error paths: All error cases tested
- Edge cases: Empty, null, boundary values

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper functions
    fn setup() -> TestFixture { }

    // Group related tests
    mod feature_a {
        #[test]
        fn test_case_1() { }
    }

    mod edge_cases {
        #[test]
        fn test_empty_input() { }
    }
}
```

### Test File Size

- If `#[cfg(test)]` module >500 LOC
- Split into `tests/` directory
- Group by feature/responsibility

---

## Rule 8: Commit Standards

### Pre-Commit Checklist (MANDATORY)

Before EVERY commit:

```bash
# 1. Check file sizes
find crates -name "*.rs" -exec wc -l {} \; | awk '$1 > 400'

# 2. Run formatter
cargo fmt --all

# 3. Run clippy (no warnings)
cargo clippy --workspace -- -D warnings

# 4. Run tests
cargo test --workspace

# 5. Check for unwrap in src (not tests)
rg "\.unwrap\(\)" crates/*/src/*.rs --glob '!*/tests.rs'
```

**If ANY check fails: DO NOT COMMIT**

### Commit Message Format

```
<type>: <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

Example:
```
refactor(memory): split session.rs into focused modules

- Created session/ directory structure
- Extracted types to types.rs (80 LOC)
- Extracted search to search.rs (120 LOC)
- Core database logic in database.rs (200 LOC)
- All tests in tests.rs (220 LOC)

Fixes file size violation (was 570 LOC, now max 220 LOC per file)
```

---

## Rule 9: Code Review Requirements

### Reviewer Responsibilities

Every PR MUST verify:

1. **File sizes**: No files exceed limits
2. **Function sizes**: No functions exceed limits
3. **God Objects**: No God Objects introduced
4. **Tests**: Comprehensive test coverage
5. **Docs**: All public items documented
6. **Errors**: Proper error handling
7. **Clippy**: Zero warnings

### Auto-Reject Criteria

**Immediately reject PR if:**
- Any file >500 LOC
- Any function >100 LOC
- Missing tests for new public API
- Clippy warnings present
- Contains `unwrap()` in production code
- Missing documentation

---

## Rule 10: Refactoring Mandate

### When to Refactor (Immediate)

**MUST refactor immediately if:**
- File exceeds 75% of size limit (300 LOC for impl files)
- Function exceeds 40 LOC
- God Object detected
- Cyclomatic complexity >10
- Code duplicated in 3+ places

### Refactoring Process

1. **Write tests first** (if not exists)
2. **Extract incrementally** (small steps)
3. **Run tests after each step**
4. **Update documentation**
5. **Delete dead code**
6. **Commit refactor separately** from new features

### Refactoring Commits

Refactoring commits MUST:
- Be separate from feature commits
- Have clear "before/after" metrics
- Pass all existing tests
- Not change behavior (only structure)

---

## Rule 11: Performance Standards

### Latency Targets (from Architecture)

```
Tier 1 Working Memory:   <1ms access
Tier 2 Session Memory:   1-10ms access
Tier 3 Knowledge Graph:  10-100ms queries
Tier 4 Archive:          100ms-1s queries
```

### Memory Limits

```
Tier 1 Working Memory:   500MB max
Total system memory:     <80GB under load (96GB available)
```

### Performance Testing

**Before committing performance-critical code:**
1. Write benchmark tests
2. Measure actual latency
3. Document performance characteristics
4. Ensure meets architecture targets

---

## Rule 12: Security Standards

### Required Practices

✅ No SQL injection (use parameterized queries)
✅ No command injection (sanitize inputs)
✅ No path traversal (validate file paths)
✅ Proper error messages (no sensitive data leaks)

### Forbidden Practices

❌ String concatenation for SQL
❌ Unsanitized user input in commands
❌ Exposing internal paths in errors
❌ Logging sensitive data

---

## Rule 13: Dependency Management

### Adding Dependencies

**Before adding ANY dependency:**
1. Check if functionality can be implemented internally
2. Verify license compatibility (must allow proprietary use)
3. Check dependency tree size
4. Verify maintenance status (last update <6 months)
5. Add to `Cargo.toml` with version constraint

### Forbidden Dependencies

❌ Unmaintained crates (>1 year no updates)
❌ Crates with security advisories
❌ GPL-licensed crates (incompatible with proprietary license)
❌ Crates with >50 transitive dependencies

---

## Rule 14: AI Assistant Specific Rules

### Before Starting Work

1. ✅ Read `STYLE_GUIDE.md`
2. ✅ Read `RULES.md` (this file)
3. ✅ Check current file sizes
4. ✅ Understand task scope
5. ✅ Plan refactoring if needed

### During Work

1. ✅ Check file size after EVERY function added
2. ✅ Refactor proactively when approaching limits
3. ✅ Extract functions when >30 LOC
4. ✅ Document as you code
5. ✅ Write tests alongside implementation

### After Work

1. ✅ Verify all files within limits
2. ✅ Run all checks (fmt, clippy, test)
3. ✅ Document what was built
4. ✅ Highlight any technical debt
5. ✅ Suggest next steps

### Reporting Violations

**If you inherit violating code:**
1. **Document the violation** clearly
2. **Propose refactoring plan**
3. **Estimate effort required**
4. **Ask user for priority** (fix now vs. later)

**Example:**
```
⚠️ STYLE VIOLATIONS DETECTED:

Files exceeding limits:
- crates/memory/src/session.rs: 570 LOC (EXCEEDS 400 LOC LIMIT by 170 LOC)
- crates/memory/src/knowledge_graph.rs: 588 LOC (EXCEEDS 400 LOC LIMIT by 188 LOC)

Recommended Refactoring:
1. session.rs → session/ directory (est. 30 min)
   - types.rs: 80 LOC
   - database.rs: 200 LOC
   - search.rs: 120 LOC
   - tests.rs: 220 LOC

2. knowledge_graph.rs → knowledge_graph/ directory (est. 45 min)
   - types.rs: 100 LOC
   - nodes.rs: 150 LOC
   - search.rs: 180 LOC
   - tests.rs: 200 LOC

Should I refactor these now before continuing? (RECOMMENDED)
```

---

## Rule 15: Continuous Improvement

### Weekly Reviews

Every week, check:
1. File size trends
2. Test coverage metrics
3. Clippy warning counts
4. Technical debt backlog

### Monthly Audits

Every month:
1. Review all files >300 LOC
2. Identify God Objects
3. Update documentation
4. Refactor high-complexity functions

---

## Appendix A: Quick Compliance Checklist

Before every commit:

- [ ] No file >400 LOC (impl) / 500 LOC (tests) / 300 LOC (modules)
- [ ] No function >50 LOC (public) / 30 LOC (private)
- [ ] No God Objects (>15 methods, >10 fields, >500 total LOC)
- [ ] All public items documented
- [ ] All tests pass: `cargo test --workspace`
- [ ] Zero clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Formatted: `cargo fmt --all`
- [ ] No `unwrap()` in production code
- [ ] All errors have `.context()`

---

## Appendix B: Violation Severity

### Critical (Block Merge)
- Files >500 LOC
- Functions >100 LOC
- God Objects
- Unwrap in production
- Missing error handling
- Failing tests

### High (Fix Before Merge)
- Files >400 LOC
- Functions >50 LOC
- Missing documentation
- Test coverage <80%
- Clippy warnings

### Medium (Technical Debt)
- Files >300 LOC
- Cyclomatic complexity >8
- Code duplication
- Suboptimal patterns

---

## Appendix C: Enforcement Tools

### Automatic Checks (CI/CD)

```yaml
# .github/workflows/quality.yml
- name: Check file sizes
  run: |
    find crates -name "*.rs" -exec wc -l {} \; | \
    awk '$1 > 500 {print "CRITICAL: " $2 " has " $1 " lines"; exit 1}'

- name: Clippy
  run: cargo clippy --workspace -- -D warnings

- name: Tests
  run: cargo test --workspace

- name: Format check
  run: cargo fmt --all -- --check
```

---

**REMEMBER: These are not guidelines. These are RULES. Follow them strictly.**
