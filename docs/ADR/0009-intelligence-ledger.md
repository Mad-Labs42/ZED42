# 5. Intelligence Ledger for Financial Governance

Date: 2026-01-21

## Status

Accepted

## Context

As ZED42 evolves into a multi-agent system, the lack of centralized financial governance poses a risk of runaway costs. Agents currently have no awareness of the fiscal impact of their LLM usage. We need a "Financial Brain" to track costs, enforce budgets, and provide data for future model routing decisions. Because financial data requires high precision and strict accountability, we cannot rely on loose logging or floating-point arithmetic.

## Decision

We will implement the **Intelligence Ledger (IL)** as a standalone, extricable library (`zed42-ledger`) that acts as the single source of truth for all token usage and costs.

### 1. Architecture

- **Crate**: `zed42-ledger` (New)
- **Dependencies**: 
    - `surrealdb` (Data Store)
    - `rust_decimal` (Financial Math)
    - `thiserror`/`anyhow` (Error Handling)
    - `serde` (Serialization)
    - `uuid` (Identity)
    - `tokio` (Async runtime)
- **Independence**: The crate will NOT depend on `zed42-agents`, `zed42-cortex`, or `zed42-blackboard`. It will define its own `EntityId`, `LeaseId`, and `Usage` types to ensure it can be extracted and used in other contexts.

### 2. Handshake Protocol: Request-Grant-Report

To prevent overspending *before* it happens, we mandate a handshake:
1.  **Request**: Entity asks for a "Lease" for an estimated cost (`request_lease`).
2.  **Grant/Deny**: Ledger checks budget. If funds exist, a `LeaseId` is reserved. usage is authorized.
3.  **Report**: After execution, the entity commits the actual usage (`commit_usage`), resolving the final cost.

### 3. Schema (SurrealDB)

- **`rate_table`**: Stores cost per 1000 tokens (input/output) for each model.
- **`budgets`**: Stores hard/soft limits for an `EntityId` (Project or Agent).
- **`ledger_entries`**: Immutable log of all Grants (Leases) and Settlements (Commits).

### 4. Data Types

- **Currency**: `rust_decimal::Decimal` (Strictly no `f64` for money).
- **TokenCount**: `u32`.
- **EntityId**: `String` or `Uuid` (Generic identifier).

### 5. Guardrails

- **Hard Cap**: Immediate rejection of `request_lease` if `budget.spent + estimated_cost > budget.hard_limit`.
- **Soft Cap**: Warning returned if `budget.spent > budget.soft_limit`, but lease is granted.

## Consequences

### Positive
- **Cost Safety**: Impossible to exceed hard budget caps if the protocol is followed.
- **Precision**: Elimination of floating-point errors in financial tracking.
- **Extricabilty**: The ledger can be reused in any Rust AI project.
- **Future Proofing**: Provides the "Cost" dimension for the future Model Orchestration Matrix (MOM).

### Negative
- **Latency**: Every LLM call now incurs a database round-trip for the lease check. (Mitigated by in-memory SurrealDB and indexing).
- **Complexity**: Agents/Tools must implement the handshake protocol, adding boilerplate to the LLM call site.
