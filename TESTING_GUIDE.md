# ZED42 Testing Guide

## Prerequisites

Ensure Rust and the required environment variables are configured:

```powershell
# Environment Variables
$env:RUSTC_WRAPPER = 'sccache'
$env:RUSTFLAGS = '--cfg surrealdb_unstable'

# Verify installation
cargo --version
sccache --version
```

## Running Tests

### All Tests
```bash
# Standard run (may use many threads)
cargo test --workspace

# Throttled run for low-spec machines
cargo test --workspace -j 1
```

### Specific Crate Tests
```bash
cargo test -p zed42-memory
cargo test -p zed42-blackboard -j 1
```

### With Output
```bash
cargo test --workspace -- --nocapture
```

## Linting with Clippy

### Check All Code
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Code Formatting

### Check Formatting
```bash
cargo fmt --all -- --check
```

### Apply Formatting
```bash
cargo fmt --all
```

## Build Verification

### Quick Check
```bash
cargo check --workspace -j 1
```

### Debug Build
```bash
cargo build --workspace -j 2
```

## Known Considerations

### 1. SurrealDB Requires Unstable Flag
The `surrealdb` crate requires the `surrealdb_unstable` flag to be passed via `RUSTFLAGS`.
```powershell
$env:RUSTFLAGS='--cfg surrealdb_unstable'
```

### 2. Sccache Speeds Up Rebuilds
If `sccache` is configured, "cold" builds after `cargo clean` will be faster as artifacts are retrieved from the global cache.

### 3. Database Tests Use In-Memory Backends
All tests use `kv-mem` for SurrealDB and temporary files for SQLite/DuckDB against temp directories. No external database server is required.

## Troubleshooting

### Issue: Type inference errors (E0282)
**Solution**: Ensure `RUSTFLAGS` is set correctly.

### Issue: Compilation freezes the system
**Solution**: Limit parallelism with `-j 1`.

### Issue: Tests fail with database lock errors
**Solution**: Run tests sequentially:
```bash
cargo test --workspace -- --test-threads=1
```

---
**Last Updated**: 2026-01-20
