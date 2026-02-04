# 6. Model Orchestration Matrix (MOM)

Date: 2026-01-21

## Status

Accepted

## Context

Agents in ZED42 currently call `zed42-llm` directly with a specific model configuration. This creates tight coupling and fragility; if OpenAI is down, the agent fails. Furthermore, strict financial governance requires a "tollgate" to check budgets before every call. We need a routing layer that handles reliability, cost-optimization, and policy enforcement transparently.

## Decision

We will implement **MOM (Model Orchestration Matrix)** as a new crate `zed42-mom`.

### 1. Architecture

- **Crate**: `zed42-mom`
- **Role**: Middleware between Agents and Providers.
- **Interface**: The `Router` struct will implement the `LlmClient` trait. This allows it to be dropped into any agent code that expects an `LlmClient`, providing instant resilience without refactoring.

### 2. Smart Escalation & Traffic Control

- **Waterfall Routing**: Each agent is assigned an `ExecutionProfile` (Tier 1, 2, 3 models).
    - If Tier 1 fails (Rate Limit / 500), automatically try Tier 2.
    - If Tier 2 fails, try Tier 3.
- **Smart Escalation**: We will modify `LlmRequest` to include `retry_cause`.
    - If `ConstrainedGen` fails validation, it re-submits the request with `retry_cause: ValidationFailure`.
    - The Router detects this and *skips* the current tier, escalating immediately to a smarter model (e.g., GPT-4o) for the retry.

### 3. Circuit Breaker

To prevent cascading failures and resource waste:
- **State**: In-memory `DashMap` tracking failures per provider/model.
- **Logic**: If 3 failures occur in 30s, the circuit opens for 5 minutes.
- **Effect**: Requests to an open circuit are immediately re-routed to the next tier without attempting a network call.

### 4. Financial Integration

The Router acts as the enforcer for `zed42-ledger`:
1.  **Intercept**: Router calculates estimated cost based on `Lease`.
2.  **Handshake**: Calls `ledger.request_lease()`.
3.  **Execute**: Calls the provider.
4.  **Settle**: Calls `ledger.commit_usage()`.
If the budget check fails, the Router denies the request before any LLM tokens are generated.

### 5. Schema

- **`model_profiles`**: Maps AgentID -> `[ModelConfig]`.
- **`routing_logs`**: Persistence for every routing decision (success, failover, escalation).

## Consequences

### Positive
- **Resilience**: Agents become immune to single-provider outages.
- **Cost Optimization**: Default to cheap models, escalate only when necessary ("Validation Bridge").
- **Observability**: Centralized logging of all AI traffic.
- **Safety**: Ledger enforcement is guaranteed by the middleware.

### Negative
- **Latency**: Additional overhead for routing logic and database lookups.
- **Complexity**: Debugging "why did it use GPT-4?" requires checking routing logs.
