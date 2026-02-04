# ADR 0003: VOX (Versatile Orchestration eXchange)

**Date:** 2026-01-21
**Status:** Accepted / Implemented

## Context
In a multi-agent system, loose coupling and strict communication contracts are essential. Legacy systems often rely on string-based messaging or unstructured JSON, which leads to runtime errors and difficult debugging. SAGA requires a type-safe, formalized protocol that governs how agents express intent and share information.

## Decision
We implemented **VOX (Versatile Orchestration eXchange)**, a type-safe communication protocol defined in `zed42-core`.

### 1. Protocol Structure
- **Core Unit:** `Message` struct.
- **Identity:** Every message has a unique `MessageId` and belongs to a `ThreadId` for conversation continuity.
- **Routing:** Uses `MessageTarget` (Team, Agent, or All).
- **Priority:** Supports `Priority` levels (0-255).

### 2. Formalized Intent (MessageType)
All agent interactions are categorized into strict variants:
- **Command:** `ExecuteTask`, `SpawnAgent`, `DissolveAgent`.
- **Query:** `RequestContext`, `QueryKnowledgeGraph`.
- **Proposal:** `ProposeSolution`, `SuggestRefactor`, `IdentifyRisk`.
- **Verdict:** `ApproveChange`, `RejectProposal`.
- **Notification:** `TaskComplete`, `MilestoneReached`, `ErrorOccurred`.

### 3. Serialization
- **Standard:** JSON via `serde`.
- **Constraint:** All message types must be serializable to ensure they can be persisted in the **MOM Reactive Substrate** and the **Memory Substrate**.

## Consequences

### Positive
- **Type Safety:** Compile-time verification of agent interactions.
- **Traceability:** ThreadIDs allow for full conversation reconstruction.
- **Integrity:** Contract-first design prevents "halting problem" communication errors.
- **Clarity:** Distinct from **IL (Intelligence Ledger)** to prevent architectural conflation.

### Negative
- **Boilerplate:** Requires defining new variants in `MessageType` for every new interaction pattern.
- **Rigidity:** Agents cannot "improvise" communication outside the protocol.
