# ADR 0006: Adversarial Multi-Agent Teams

**Date:** 2026-01-21
**Status:** Accepted / Implemented

## Context
Standard multi-agent systems often suffer from "conformation bias" where agents simply agree with each other or follow instructions without critical evaluation. To produce FAANG+ quality code, ZED42 requires a system of checks and balances that mimics a high-performance engineering organization.

## Decision
We implemented a **Hierarchical Adversarial Multi-Agent Architecture** consisting of three specialized teams with opposing mandates.

### 1. The Red Team (Offensive)
- **Mandate:** Identify weaknesses, edge cases, security vulnerabilities, and performance bottlenecks.
- **Incentive:** Rewarded for finding flaws BEFORE code reaches production.
- **Role:** Challenges Blue Team proposals with reproduction cases.

### 2. The Blue Team (Defensive)
- **Mandate:** Implement features safely with comprehensive testing and backward compatibility.
- **Incentive:** Rewarded for passing all Red Team challenges and Green Team review.
- **Role:** The primary "builder" group responsible for feature delivery.

### 3. The Green Team (Governance)
- **Mandate:** Enforce long-term architectural integrity, arbitrate disputes, and maintain technical standards.
- **Incentive:** Rewarded for preventing technical debt and maintaining codebase health.
- **Role:** The final "judge" that must approve code merges and architectural changes.

## Protocol: The Adversarial Handshake
1. **Blue Team** proposes an implementation (via VOX/MOM).
2. **Red Team** is spawned to "attack" the implementation (e.g., Fuzzing, Static Analysis).
3. **Green Team** arbitrates any disputes and validates the final artifacts against the `constraints.toml`.

## Consequences

### Positive
- **High Integrity:** Code is battle-tested by an automated adversary before human review.
- **Separation of Concerns:** Agents have clear, focused mandates.
- **Scalability:** Teams can be scaled independently based on task complexity.

### Negative
- **Latency:** Adversarial loops add significant time to the OODA cycle.
- **Token Usage:** Coordination between three teams is more expensive than a single-agent approach.
- **Complexity:** Requires sophisticated state management via the **Blackboard**.
