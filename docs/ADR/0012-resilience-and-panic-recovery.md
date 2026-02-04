# ADR 0012: Agent Resilience and Failure Recovery - StateGuard Pattern

## Status
Proposed/Accepted (Implemented)

## Context
In a high-concurrency, long-running agentic system like Zed 42, individual agent logic may encounter unexpected panics (e.g., during complex LLM result parsing) or early returns.
Without a robust recovery mechanism:
1. **Ghost States**: Agents could remain in a `Working` status indefinitely in the Blackboard, preventing further task assignments.
2. **Resource Leaks**: Database connections or file handles might remain open.
3. **Observability**: A crash without a state revert leaves no forensic trace of the failure in the system-level status.

## Decision
Implement the **`StateGuard` Pattern** (RAII for Agent State) in `crates/agents/src/state.rs`.

### 1. RAII State Reversion
The `StateGuard` struct holds a reference to the agent's identity and status. It is initialized when an agent begins a high-risk operation (e.g., the OODA cycle).
- **Behavior**: If the agent's execution thread panics or finishes without explicitly marking completion, the `Drop` implementation of `StateGuard` automatically reverts the agent's status to `Idle` (or `Failed` if a panic occurred).

### 2. Zero-Panic Compliance
Strictly wrap all fallible operations in the cognitive loop with `anyhow::Context`.
- **Forensic Visibility**: Every potential failure point must include a context string that describes the operation being attempted, allowing for rapid debugging of "failed" states.

### 3. Isolation
Combine `StateGuard` with the **Titan Substrate** handles. Since handles are clones of `Arc<RwLock<T>>`, a panic in one agent does not poison the underlying database connection or crash other agents in the same workspace.

## Consequences
- **Positive**: Guaranteed state consistency. Agents never remain "stuck" in a working state after a crash.
- **Positive**: Improved reliability for automated team management (the Team Manager can safely re-spawn agents that enter a `Failed` state).
- **Negative**: Marginal overhead for status updates on the Blackboard.
- **Neutral**: Requires all new agent implementations to wrap their entry points with a `StateGuard`.
