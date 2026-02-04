# ADR 0004: SurrealDB Integration Strategy and Type Enforcement

**Date:** 2026-01-20
**Status:** Accepted / Implemented

## Context
Integrating `SurrealDB` (v1.5.6) into the ZED42 Rust workspace presented significant challenges:
1.  **Type Inference Failures**: The Rust compiler (E0282) struggled to infer types for SurrealDB's async `.create()` and `.query()` builder chains, leading to persistent build failures.
2.  **Feature Flag Conflicts**: The `surrealdb` crate has unstable features (like `kv-surrealkv`) that require special configuration flags (`--cfg surrealdb_unstable`) which cannot be set directly in `Cargo.toml`.
3.  **Backend Locking**: Using persistent backends (`kv-rocksdb`) caused file locking issues during rapid development iterations and test runs.

## Decision
We established a strict **SurrealDB Integration Protocol** to ensure stability and compilation correctness.

### 1. The "Nuclear" Type Annotation Strategy
We mandated explicit type bindings for *every* SurrealDB interaction. Instead of relying on method chaining inference, we split declarations and explicitly type the variables.

**Rejected Pattern (Implicit):**
```rust
// Fails compilation
let res = db.create("thing").content(data).await?;
```

**Adopted Pattern (Explicit):**
```rust
// Compiles reliably
let _: Vec<Record> = db
    .create("thing")
    .content(data)
    .await?;
```
This forces the compiler to know exactly what the expected return type is before the `await` future resolves.

### 2. Environment-Based Feature Flags
We use `RUSTFLAGS` to enable unstable internal features without polluting the `Cargo.toml` dependencies manifest.
-   **Action**: Set `$env:RUSTFLAGS='--cfg surrealdb_unstable'` in the development shell / global profile.
-   **Reason**: Keeps the standard build clean while unlocking necessary advanced features for the developer.

### 3. In-Memory Development Backend
-   **Decision**: Use `kv-mem` (SurrealDB's in-memory engine) for all development and testing databases.
-   **Reason**: eliminates file locking issues, dramatically speeds up tests (no disk I/O), and ensures a clean state on every restart. Persistence is reserved for specific `prod` profiles or future migrations.

## Consequences

### Positive
-   **Build Stability**: The "Nuclear" typing eliminates vague E0282 errors.
-   **Dev Velocity**: `kv-mem` allows for instant initialization and teardown of complex graph databases.
-   **Clarity**: Explicit typing makes the code self-documenting regarding what data structures are expected from the DB.

### Negative
-   **Verbosity**: Code is more verbose.
-   **Data Persistence**: Dev data is lost on restart (by design, but requires awareness).

## Compliance
This decision aligns with the "Stabilizing Rust Crates" objective and the "Zero-Waste" infrastructure goal by minimizing persistent disk artifacts during development.
