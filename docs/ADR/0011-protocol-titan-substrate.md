# ADR 0011: Protocol Titan Substrate - Hardware-Locked Industrial Architecture

## Status
Proposed/Accepted (Implemented)

## Context
As Zed 42 moved from prototype to industrial-scale execution, the original "field-injected" dependency model for agents (where each agent held direct `Arc<Database>` fields) became brittle. 
1. **Concurrency Risk**: Direct field holdings led to ambiguous lock boundaries during complex OODA cycles.
2. **Hardware Fragility**: High-velocity agent operations (file writes, LLM indexing) posed a risk of disk exhaustion (OS Error 112) without central backpressure.
3. **Data Integrity**: Non-atomic filesystem operations could leave the codebase in a corrupted state if an agent crashed or was terminated mid-write.

## Decision
Implement **Protocol Titan**, a unified substrate layer that abstracts hardware and state management away from individual agents.

### 1. The Titan Registry (`TitanSubstrate`)
A centralized, thread-safe registry (`crates/core/src/titan.rs`) that manages all system-wide handles.
- **Pattern**: `Arc<RwLock<T>>` for all subsystems (Blackboard, Memory, LLM).
- **Access**: Agents retrieve non-blocking handles via generic getters, ensuring locks are local and short-lived.

### 2. SpaceSentry (Hardware Backpressure)
A hardware-aware health monitor integrated into the Substrate.
- **Critical Threshold**: 500MB availability on the host C:\ drive.
- **Action**: Signals a backpressure error that halts agent cognitive loops and denies FS write requests.

### 3. FileStateGuard (Atomic Shadow Writes)
A mandatory filesystem guard utilizing the "Shadow Write" pattern.
- **Mechanism**: All writes occur to `.tmp` sibling files.
- **Atomicity**: Changes only commit to the target path via an atomic rename upon success.
- **RAII**: Incomplete or failed operations auto-cleanup temporary garbage on `Drop`.

### 4. Cortex (Substrate-Driven Brain)
Refactored the Agent Orchestrator into the **Cortex**. 
- **Evolution**: The Cortex no longer "owns" its databases. It "subscribes" to handles from the Titan Substrate.
- **Self-Healing**: Integrated with `StateGuard` for panic recovery and `SpaceSentry` for hardware-driven pausing.

## Consequences
- **Positive**: Increased system stability, zero data corruption on write, and proactive hardware protection.
- **Positive**: Improved developer ergonomics (agent constructors are now simplified to a single `substrate` injection).
- **Negative**: Slight overhead for handle acquisition (mitigated by `parking_lot` performance).
- **Neutral**: Requires all new toolboxes to strictly follow the `FileStateGuard` pattern for compliance.
