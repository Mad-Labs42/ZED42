# ADR 0002: Adoption of Wide & Flat Workspace Architecture

**Date:** 2026-01-20
**Status:** Accepted / Implemented

## Context
The initial ZED42 codebase faced several structural challenges typical of growing Rust projects:
1.  **Monolithic Coupling**: Logic for agents, memory, and orchestration was intertwined, making it difficult to isolate features or test components independently.
2.  **Circular Dependencies**: Type definitions scattered across modules led to potential import cycles (e.g., Agents needing Blackboard types, Blackboard needing Agent types).
3.  **Compilation Bottlenecks**: A single large crate meant any change triggered a full recompile of the application logic, slowing down the "edit-compile-test" loop.

## Decision
We adopted a **Wide & Flat Cargo Workspace** architecture. This strategy involves splitting the codebase into multiple focused crates residing in a `crates/` directory, all managed by a root `Cargo.toml`.

**Key Structural Actions:**
1.  **`zed42-core`**: A foundational leaf-node crate containing all shared types (`AgentId`, `Message`, `SessionId`), traits (`AgentBehavior`), and error types. It has *zero* internal dependencies on other project crates.
2.  **Specialized Crates**:
    -   `zed42-memory`: Handles storage tiers. Depends on `core`.
    -   `zed42-blackboard`: Handles message passing. Depends on `core`.
    -   `zed42-agents`: Defines agent logic. Depends on `core` (and others as needed, but strictly acyclic).
    -   `zed42-cortex`: The orchestrator. Depends on all of the above.
3.  **Dynamic Dispatch**: The Cortex uses `Box<dyn AgentBehavior>` to manage agents, decoupling the orchestrator from specific agent implementations.

## Detailed Implementation
-   **Root `Cargo.toml`**: Defines the workspace members (`crates/*`) and shared dependency versions to ensure consistency.
-   **Directory Layout**:
    ```
    root/
    ├── Cargo.toml (Workspace)
    ├── crates/
    │   ├── core/ (Types & Traits)
    │   ├── memory/ (Database Logic)
    │   ├── blackboard/ (Pub/Sub)
    │   ├── agents/ (Business Logic)
    │   ├── cortex/ (Orchestrator)
    │   └── ui/ (Frontend)
    ```

## Consequences

### Positive
-   **Decoupling**: Strict dependency graph prevents circular references. `core` acts as the single source of truth for types.
-   **Build Performance**: Cargo can compile crate dependencies in parallel. Modifying `zed42-agents` does not require recompiling `zed42-memory`.
-   **Testability**: Each crate has its own test suite (`cargo test -p name`), allowing for focused verification.
-   **Maintainability**: Logical boundaries are enforced by the compiler. You cannot import a type if the dependency isn't declared in `Cargo.toml`.

### Negative
-   **Boilerplate**: Requires managing multiple `Cargo.toml` files.
-   **Verbosity**: Re-exporting types from `core` can sometimes feel repetitive.

## Compliance
This structure facilitates the "Systemic Audit" requirements and supports the agentic workflow where specific agents can be assigned to different parts of the system without stepping on each other's toes.
