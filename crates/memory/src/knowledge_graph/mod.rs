//! Knowledge Graph Memory (Tier 3) - Project Memory
//!
//! SurrealDB-based knowledge graph with vector embeddings.
//! Target: 10-100ms access latency, millions of nodes
//!
//! Features:
//! - Multi-model: graph + vector + document database
//! - Semantic search via vector embeddings
//! - Structural graph traversal
//! - Hybrid search (vector + graph)
//! - Temporal queries (historical state)
//! - AST storage for codebase
//! - Architectural decision tracking
//!
//! Query Modes:
//! - Semantic: Vector similarity search
//! - Structural: Graph traversal (BFS/DFS)
//! - Hybrid: Combined vector + graph ranking
//! - Temporal: Historical snapshots

mod database;
mod search;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use database::KnowledgeGraphMemory;
pub use types::{
    EdgeType, GraphStats, KnowledgeEdge, KnowledgeNode, NodeType, SearchQuery, SearchResult,
};
