---
trigger: always_on
---

# ZED42 Code Style Guide and Standards

**Version**: 1.0
**Last Updated**: January 19, 2025
**Status**: Mandatory - All code must comply

---

## Core Philosophy

**ZED42 must be built to the same standards it enforces.**

This means:
- No God objects
- FAANG+ code quality
- Industry-leading architecture
- Production-ready from day one
- Self-consistent enforcement

---

## 1. File Size Limits

### HARD LIMITS (Never Exceed)

| File Type | Maximum Lines | Rationale |
|-----------|--------------|-----------|
| **Module files (lib.rs, mod.rs)** | 300 LOC | Coordination only, not implementation |
| **Implementation files** | 400 LOC | Single responsibility enforcement |
| **Test modules** | 500 LOC | Tests may be verbose but grouped |
| **Configuration files** | 200 LOC | Keep config simple and focused |

### When approaching limits:

1. **Split by responsibility** - Create separate modules
2. **Extract to submodules** - Use `mod submodule;` pattern
3. **Group related functionality** - Create focused feature modules

### Examples:

❌ **BAD**: `session.rs` with 570 lines (tests + impl + types)
```
session.rs (570 LOC) - TOO LARGE
├── Types (50 LOC)
├── Implementation (300 LOC)
└── Tests (220 LOC)
```

✅ **GOOD**: Split into focused modules
```
session/
├── mod.rs (30 LOC) - Public API only
├── types.rs (50 LOC) - SessionEntry, EntryType, etc.
├── database.rs (150 LOC) - DB operations
├── search.rs (100 LOC) - FTS5 search logic
└── tests.rs (220 LOC) - All tests
```

---

## 2. Function Size Limits

### HARD LIMITS

| Function Type | Maximum Lines | Maximum Complexity |
|---------------|--------------|-------------------|
| **Public API functions** | 50 LOC | Cyclomatic: 10 |
| **Private helper functions** | 30 LOC | Cyclomatic: 8 |
| **Test functions** | 60 LOC | Cyclomatic: 5 |
| **Constructors (`new`, `from`, etc.)** | 40 LOC | Cyclomatic: 6 |

### Enforcement:

- Use `cargo clippy` with these lints:
  ```toml
  [lints.clippy]
  too_many_lines = "warn"
  cognitive_complexity = "warn"
  ```

### When a function is too large:

1. **Extract helper functions** - Private functions in same file
2. **Extract to separate module** - If helpers are reusable
3. **Use builder pattern** - For complex construction
4. **Simplify logic** - Question if complexity is necessary

### Example:

❌ **BAD**: 80-line function with complex logic
```rust
pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SessionEntry>> {
    // 80 lines of complex query building, execution, and mapping
}
```

✅ **GOOD**: Split into focused functions
```rust
pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SessionEntry>> {
    let stmt = self.prepare_search_statement()?;
    let rows = self.execute_search(&stmt, query, limit)?;
    self.map_search_results(rows)
}

fn prepare_search_statement(&self) -> Result<Statement> { /* ... */ }
fn execute_search(&self, stmt: &Statement, query: &str, limit: usize) -> Result<Rows> { /* ... */ }
fn map_search_results(&self, rows: Rows) -> Result<Vec<SessionEntry>> { /* ... */ }
```

---

## 3. Module Organization

### Directory Structure Rules

Every module with >2 implementation files MUST use directory structure:

```
module_name/
├── mod.rs          # Public API exports only (30-50 LOC max)
├── types.rs        # Data structures and enums
├── implementation/ # Implementation details (if complex)
│   ├── mod.rs
│   ├── core.rs
│   └── helpers.rs
└── tests.rs        # All tests (or tests/ directory)
```

### Module File Responsibilities

| File | Purpose | Max LOC | Contains |
|------|---------|---------|----------|
| `mod.rs` | Public API surface | 50 | `pub use`, module declarations, re-exports |
| `types.rs` | Data structures | 200 | Structs, enums, type aliases |
| `impl.rs` or `core.rs` | Main implementation | 300 | Primary functionality |
| `helpers.rs` | Private utilities | 200 | Internal helper functions |
| `tests.rs` | Test suite | 500 | All unit tests |

### Real Example - Refactor session.rs:

**Current (VIOLATES STANDARDS):**
```
crates/memory/src/session.rs (570 LOC) ❌
```

**Required Structure:**
```
crates/memory/src/session/
├── mod.rs (40 LOC)           # pub use types::*; pub use database::SessionMemory;
├── types.rs (80 LOC)         # SessionEntry, EntryType, SessionStats
├── database.rs (200 LOC)     # SessionMemory impl (core operations)
├── search.rs (120 LOC)       # FTS5 search implementation
└── tests.rs (220 LOC)        # All tests
```

---

## 4. Rust-Specific Standards

### Error Handling

✅ **REQUIRED:**
- Always use `Result<T, E>` for fallible operations
- Use `anyhow::Result` for application code
- Use `thiserror` for library errors
- Always add `.context()` for error chain

❌ **FORBIDDEN:**
- `unwrap()` in production code (only in tests)
- `expect()` without detailed message
- Silently ignoring errors

### Documentation

✅ **REQUIRED for all public items:**
```rust
/// Brief one-line summary
///
/// Detailed explanation of what this does.
///
/// # Arguments
/// - `param1` - Description
/// - `param2` - Description
///
/// # Returns
/// Description of return value
///
/// # Errors
/// When this function returns an error
///
/// # Examples
/// ```
/// let result = function(arg1, arg2)?;
/// ```
pub fn function(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // implementation
}
```

### Type Design

✅ **REQUIRED:**
- Use newtype pattern for domain types
- Derive `Debug, Clone, Serialize, Deserialize` when applicable
- Make fields private, provide methods
- Use builder pattern for >3 constructor args

❌ **FORBIDDEN:**
- Public fields on structs (except for simple DTOs)
- Stringly-typed data (use enums)
- Primitive obsession (wrap primitives)

---

## 5. No God Objects Rule

### Definition

A "God Object" is a struct/class that:
- Has >15 methods
- Knows about >5 other modules
- Has >10 fields
- Exceeds 500 total LOC including impl blocks

### Detection

Run this check before committing:
```bash
# Count methods in impl block
rg "fn \w+\(" crates/*/src/*.rs -A 1 | wc -l

# Check file size
find crates -name "*.rs" -exec wc -l {} \; | awk '$1 > 400 {print}'
```

### Enforcement Strategy

When approaching God Object territory:

1. **Split by responsibility**
   - Extract cohesive groups of methods
   - Create focused submodules

2. **Use composition over inheritance**
   - Delegate to smaller, focused structs
   - Use traits for polymorphism

3. **Apply SOLID principles**
   - Single Responsibility
   - Open/Closed
   - Liskov Substitution
   - Interface Segregation
   - Dependency Inversion

### Example Refactor:

❌ **BAD**: God Object
```rust
// 600 LOC file with 25 methods
pub struct SessionMemory {
    conn: Connection,
    session_id: Uuid,
    db_path: PathBuf,
    // ... 10 more fields
}

impl SessionMemory {
    // 25 methods handling:
    // - Database connection
    // - Schema management
    // - CRUD operations
    // - Full-text search
    // - Statistics
    // - Migrations
    // - Backup/restore
    // - etc.
}
```

✅ **GOOD**: Composed objects
```rust
pub struct SessionMemory {
    database: SessionDatabase,
    search: SessionSearch,
    stats: SessionStats,
}

struct SessionDatabase {
    conn: Connection,
    schema: SchemaManager,
}

struct SessionSearch {
    fts_engine: Fts5Engine,
}

struct SessionStats {
    metrics: MetricsCollector,
}
```

---

## 6. Testing Standards

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper functions at top
    fn setup_test_env() -> TestEnv { /* ... */ }

    // Group related tests
    mod basic_operations {
        #[test]
        fn test_insert() { /* ... */ }

        #[test]
        fn test_get() { /* ... */ }
    }

    mod edge_cases {
        #[test]
        fn test_empty_input() { /* ... */ }

        #[test]
        fn test_invalid_data() { /* ... */ }
    }
}
```

### Test Coverage Requirements

- **Public API functions**: 100% coverage
- **Private functions**: >80% coverage
- **Error paths**: Must test all error cases
- **Edge cases**: Empty, null, boundary values

### Test File Size

- If `tests` module >500 LOC, split into `tests/` directory:
  ```
  module_name/
  ├── mod.rs
  ├── implementation.rs
  └── tests/
      ├── mod.rs
      ├── basic_tests.rs
      ├── integration_tests.rs
      └── edge_cases.rs
  ```

---

## 7. Pre-Commit Checklist

Before committing ANY code, verify:

- [ ] No file exceeds size limits (see Section 1)
- [ ] No function exceeds 50 LOC (see Section 2)
- [ ] No God Objects detected (see Section 5)
- [ ] All public items documented (see Section 4)
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Formatted: `cargo fmt --all`
- [ ] No `unwrap()` in production code
- [ ] All errors have `.context()`

---

## 8. Refactoring Guidelines

### When to Refactor

**Immediate refactor required when:**
- File exceeds LOC limits
- Function exceeds complexity limits
- God Object detected
- Cyclomatic complexity >10

**Schedule refactor when:**
- Test coverage <80%
- Similar code in 3+ places (DRY violation)
- Module has >7 dependencies

### How to Refactor

1. **Write tests first** - Ensure behavior preserved
2. **Extract incrementally** - Small, verifiable steps
3. **Run tests after each step** - Catch regressions immediately
4. **Update documentation** - Keep docs in sync
5. **Delete dead code** - Remove unused functions

---

## 9. Code Review Standards

### Reviewer Checklist

All PRs must verify:

- [ ] File sizes within limits
- [ ] Function complexity within limits
- [ ] No God Objects introduced
- [ ] Comprehensive tests added
- [ ] Documentation complete
- [ ] Error handling proper
- [ ] No clippy warnings

### Automatic Rejection Criteria

PRs will be **automatically rejected** if:
- Any file >500 LOC
- Any function >100 LOC
- Missing tests for new public functions
- Clippy errors present
- Unwrap in production code

---

## 10. Tools and Automation

### Required Tools

```bash
# Install
cargo install cargo-clippy cargo-fmt cargo-audit

# Pre-commit hook (add to .git/hooks/pre-commit)
#!/bin/bash
cargo fmt --all -- --check || exit 1
cargo clippy --workspace -- -D warnings || exit 1
cargo test --workspace || exit 1
```

### Recommended VS Code Settings

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "files.trimTrailingWhitespace": true
}
```

---

## 11. Enforcement Priority

### Critical (Must Fix Immediately)

1. Files >500 LOC
2. Functions >100 LOC
3. God Objects
4. Missing error handling
5. Unwrap in production code

### High Priority (Fix Before Merge)

1. Files >400 LOC
2. Functions >50 LOC
3. Missing documentation
4. Test coverage <80%
5. Clippy warnings

### Medium Priority (Technical Debt)

1. Files >300 LOC
2. Cyclomatic complexity 8-10
3. Code duplication
4. Suboptimal patterns

---

## Appendix: Quick Reference

### File Size Limits
- Module files: 300 LOC
- Implementation: 400 LOC
- Tests: 500 LOC

### Function Limits
- Public: 50 LOC, complexity 10
- Private: 30 LOC, complexity 8

### God Object Indicators
- >15 methods
- >10 fields
- >500 total LOC

### Required for All Public Items
- Rustdoc comments
- Examples in docs
- Error documentation
- Test coverage

---

**Remember: Build ZED42 to the standards it enforces. No exceptions.**
