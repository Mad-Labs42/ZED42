# ADR 0005: Aura Vitality Substrate

**Date:** 2026-01-21
**Status:** Accepted / Implemented

## Context
As ZED42 transitions to a multi-agent orchestration system, the risk of "Ghost Agents" (processes that have crashed or hung but still appear active in the database) became a critical failure point. A robust, 4-tiered vitality system is required to ensure that the system state reflects reality and that resources are reclaimed autonomously when agents fail.

## Decision
We implemented **AURA (Blackboard Vitality Substrate)**, a tiered hierarchy of health monitoring and reactive recovery.

### Tier 1: Pulse (Persistence)
- **Mechanism:** Every active agent is mandated to send a "Pulse" to the `agent_heartbeat` table in the Blackboard.
- **Implementation:** USES SurrealDB 2.x native `UPSERT` with `time::now()`.
- **Infrastructure:** The `agent_heartbeat` table is `SCHEMAFULL` with automated `TTL` cleanup events for legacy data.

### Tier 2: Sentinel (Observation)
- **Mechanism:** A background monitoring process in the Blackboard crate that categorizes agent health based on the time since their last pulse.
- **States:**
  - **Healthy:** Pulse received within `< 60s`.
  - **Laggard:** No pulse for `> 60s`. FLAG set in DB.
  - **Zombie:** No pulse for `> 300s` (5 minutes).

### Tier 3: Reactive Broadcast (Notification)
- **Mechanism:** When an agent's state transitions to Laggard or Zombie, the Sentinel issues a critical broadcast via the **MOM Reactive Substrate**.
- **Payload:** `DissolveAgent` or `StatusAlert` messages are pushed with high priority (255) to all relevant teams.

### Tier 4: Reaper (Auto-Correction)
- **Mechanism:** The Sentinel autonomously performs the dissolution of Zombie records to reclaim system resources and prevent stale decision-making.
- **Traceability:** Every reap event is logged in the `routing_logs` (via MOM) to provide an audit trail for human supervisors.

## Consequences

### Positive
- **Reliability:** Eliminates Ghost Agents from the orchestration logic.
- **Efficiency:** Automatic cleanup of stale database records.
- **Observability:** Real-time visibility into agent health across the entire team.

### Negative
- **Network Traffic:** Consistent (though small) overhead for heartbeat pulses.
- **Complexity:** Requires background management loops within the Blackboard service.
