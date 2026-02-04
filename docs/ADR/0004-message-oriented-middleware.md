# ADR 0004: Message-Oriented Middleware (MOM)

**Date:** 2026-01-21
**Status:** Accepted / Implemented

## Context
A multi-agent system requires a high-performance, reactive communication backbone. Simple database polling is too slow and resource-intensive, while traditional message brokers (like RabbitMQ) add significant operational complexity. We need a solution that leverages our existing database (SurrealDB) but provides native, sub-millisecond reactivity.

## Decision
We implemented **MOM (Message-Oriented Middleware)** using SurrealDB 2.x's native `LIVE SELECT` functionality over WebSockets.

### 1. The Reactive Bus
- **Technology:** SurrealDB 2.x `LIVE SELECT`.
- **Interface:** `BlackboardWatcher` in `zed42-blackboard`.
- **Connectivity:** Maintaining a resilient WebSocket connection with exponential backoff.

### 2. Team-Based Routing
- **Broadcasting:** Notifications from the `blackboard` table are routed into team-specific (`Red`, `Blue`, `Green`) broadcast channels.
- **Filtering:** Agents only subscribe to messages relevant to their mandate, preventing context pollution.

### 3. Backpressure and Latency
- **Dropping:** If an agent lags behind the broadcast channel's buffer (1024 messages), the MOM will drop messages for that agent to protect system stability.
- **Latency:** Real-time push notifications eliminate polling latency, enabling `< 10ms` observe-to-orient transitions.

## Consequences

### Positive
- **Simplicity:** No external message broker required; SurrealDB acts as both the store and the bus.
- **Performance:** Native C++ implementation of LIVE SELECT in SurrealDB provides massive throughput.
- **Reliability:** Integrated with the **AURA Vitality Substrate** for system health monitoring.

### Negative
- **WebSocket Dependency:** Requires stable WS connections; handle-able via the resilient `watcher.rs` loop.
- **Consistency:** `LIVE SELECT` is eventually consistent compared to the ACID properties of the database state.
