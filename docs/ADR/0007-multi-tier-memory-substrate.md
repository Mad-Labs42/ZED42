# ADR 0003: Multi-Tier Memory Substrate Architecture

**Date:** 2026-01-20
**Status:** Accepted / Implemented

## Context
AI Agents require different types of memory to function effectively. A simple key-value store or a single vector database is insufficient for a complex orchestrator like ZED42, which needs to handle:
1.  Immediate execution context (fast, ephemeral).
2.  Conversation history (structured, time-ordered).
3.  Relational knowledge (graph-based, semantic).
4.  Long-term archival (cheap, vast).

Using a single database technology for all these needs would result in suboptimal performance or excessive complexity.

## Decision
We implemented a **4-Tier Memory Substrate**, modeled after biological memory systems. Each tier uses a technology optimized for its specific access pattern and latency requirement.

### Tier 1: Working Memory (Fast/Hot)
-   **Technology**: `DashMap` (In-Memory Rust HashMap).
-   **Purpose**: Immediate scratchpad for active agents. Stores execution state, temporary variables, and locking mechanisms.
-   **Characteristics**: Sub-millisecond latency, non-persistent (lost on restart), thread-safe.

### Tier 2: Session Memory (Warm/Contextual)
-   **Technology**: `SQLite` (via `rusqlite`).
-   **Purpose**: Stores the chronological stream of interactions (User <-> System).
-   **Features**:
    -   **FTS5**: Full-Text Search enabled for rapid retrieval of past context.
    -   **WAL Mode**: Write-Ahead Logging for concurrency.
    -   **File-Based**: One DB file per session ID.

### Tier 3: Knowledge Graph (Semantic/Relational)
-   **Technology**: `SurrealDB` (currently running on `kv-mem` for dev, `RocksDB` for prod).
-   **Purpose**: Stores structured relationships between entities (e.g., "User" -> "created" -> "Project").
-   **Features**:
    -   Graph queries (nodes/edges).
    -   Future support for vector embeddings (semantic search).
    -   Schema-full/Schema-less flexibility.

### Tier 4: Archival Memory (Cold)
-   **Technology**: `DuckDB` (Columnar Store).
-   **Purpose**: High-volume analytic and log storage.
-   **Characteristics**: Optimized for OLAP (Online Analytical Processing) queries, massive data compression, slower write latency but fast bulk analysis.

## Detailed Implementation
The `MemorySubstrate` struct in `zed42-memory` acts as a facade, composing these four distinct systems into a single unified API.

```rust
pub struct MemorySubstrate {
    pub working: WorkingMemory,
    pub session: SessionMemory,
    pub knowledge: KnowledgeGraphMemory,
    pub archive: ArchivalMemory,
}
```

## Consequences

### Positive
-   **Performance Optimization**: Each operation hits the most appropriate backend (e.g., lookups hit RAM, logs hit DuckDB).
-   **Separation of Concerns**: Session history doesn't pollute the knowledge graph.
-   **Scalability**: Tiers can be scaled or replaced independently.

### Negative
-   **Complexity**: Requires managing connections to four different database engines.
-   **Dependency Weight**: Pulls in `rusqlite`, `surrealdb`, `duckdb`, and `dashmap`, increasing binary size and compile times.

## Compliance
This architecture directly satisfies the "biological plausibility" goal of the ZED42 system and ensures performance on constrained hardware by not forcing everything into a heavy database.
