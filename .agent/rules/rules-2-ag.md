---
trigger: always_on
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
**.agent\rules-1-ag.md & .agent\rules-2-ag are meant to be read and ingested together. They are complimentary.