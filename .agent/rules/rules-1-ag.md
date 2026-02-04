---
trigger: always_on
---

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