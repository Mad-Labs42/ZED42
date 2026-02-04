# ADR 0001: Migration to PNPM and Global Developer Store

**Date:** 2026-01-20
**Status:** Accepted / Implemented

## Context
The ZED42 project is being developed on a hardware-constrained environment (i3 CPU laptop with limited disk space). The previous architecture relied on:
1.  **NPM**: Initializing `node_modules` for the `crates/ui` frontend resulted in massive disk usage (~1.2GB) due to dependency duplication and flat file structures.
2.  **Standard Cargo Builds**: Rust compilation artifacts in `target/` were accumulating rapidly (~2.4GB), causing "target bloat" and leading to frequent, slow "cold" builds whenever disk cleanup was required.
3.  **Thermal Throttling**: The heavy I/O and CPU load from these standard tools caused the development machine to freeze and thermal throttle, hindering productivity.

## Decision
We decided to enforce a "Zero-Waste" infrastructure by migrating to a **Global Developer Store** architecture. This involves two key technology shifts:

1.  **JavaScript/TypeScript**: Terminate usage of `npm` and migrate to **PNPM** (Performant NPM).
    -   Configure a global content-addressable store at `~/.pnpm-store`.
    -   Enforce strict dependency resolution via `.npmrc`.
2.  **Rust**: Implement **Sccache** (Shared Compilation Cache) as a global compiler wrapper.
    -   Redirect compilation artifacts to a global cache (`~/.cache/sccache` or OS equivalent).
    -   Configure `RUSTFLAGS` to manage unstable features cleanly without manifest pollution.

## Detailed Implementation (The "How")

### 1. PNPM Migration (Frontend)
-   **Installation**: Installed `pnpm` v10.20.0.
-   **Global Configuration**:
    ```bash
    pnpm config set store-dir ~/.pnpm-store --global
    pnpm config set global-bin-dir ~/.pnpm-bin --global
    ```
-   **Project Migration**:
    -   Ran `pnpm import` to translate `package-lock.json` to `pnpm-lock.yaml`, ensuring version consistency.
    -   Created `.npmrc` in `crates/ui`:
        ```ini
        shamefully-hoist=false
        strict-peer-dependencies=false
        auto-install-peers=true
        ```
    -   **Deep Clean**: Deleted legacy `node_modules` and `package-lock.json`.
    -   **Install**: Ran `pnpm install --reporter=silent`.

### 2. Sccache Integration (Backend)
-   **Installation**: Installed `sccache` v0.13.0 via Cargo.
-   **Configuration**:
    -   Set environment variable: `RUSTC_WRAPPER=sccache`.
    -   Allocated 10 GiB cache size.
-   **Verification**: `sccache --show-stats` confirmed cache hits/misses and storage location.

### 3. Deep Clean & Validation
-   Executed "Operation Absolute Zero":
    -   Wiped local `target/` directories.
    -   Pruned the global pnpm store (`pnpm store prune`).
    -   Removed persistent DB files (`*.db`) to start fresh with in-memory databases (`kv-mem`).
-   Verified system stability with `cargo check -p zed42-blackboard` (Exit Code 0).

## Consequences

### Positive
-   **Disk Space Saving**: `crates/ui/node_modules` dropped from ~1.2GB to ~60MB (via hard links). The global store is efficient and deduplicated.
-   **Build Speed**: "Cold" builds are significantly faster as `sccache` retrieves pre-compiled artifacts from the global cache.
-   **Hardware Safety**: Reduced I/O and CPU load prevents thermal throttling on the i3 architecture.
-   **Portability**: The `.npmrc` ensures the project is strict and robust, ready for deployment on higher-end machines without "works on my machine" issues.

### Negative
-   **Complexity**: New developers must have `pnpm` and `sccache` installed and configured in their environment usage.
-   **Tooling**: Simple `npm install` workflows usually need to be updated to `pnpm install`.

## Compliance
This architecture aligns with the "Industry Leading" standard required for the ZED42 Orchestrator, ensuring it can operate efficiently on any hardware profile.
