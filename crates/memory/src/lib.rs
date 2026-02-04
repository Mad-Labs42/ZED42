//! ZED42 Memory Substrate - Four-Tier Memory Hierarchy
//!
//! Copyright (c) 2025 ZED42 Team. All rights reserved.
//! This software is proprietary. See LICENSE file for terms.
//!
//! Tier 1: Working Memory (Hot Cache) - In-memory Rust data structures
//! Tier 2: Session Memory (Warm Storage) - SQLite with FTS5
//! Tier 3: Project Memory (Knowledge Graph) - SurrealDB with vector embeddings
//! Tier 4: Archive Memory (Cold Storage) - Parquet + DuckDB

use anyhow::{Context, Result};
use duckdb::OptionalExt;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use zed42_llm::LlmClient;

pub mod archive;
pub mod knowledge_graph;
pub mod session;
pub mod working;

use archive::{ArchiveMemory, ArchiveQuery};
use knowledge_graph::{KnowledgeGraphMemory, SearchQuery};
use session::SessionMemory;
pub use working::{WorkingMemory, CacheStats};
use zed42_core::types::SessionId;


/// Memory tier enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryTier {
    Working,  // Hot cache, <1ms
    Session,  // Warm storage, 1-10ms
    Project,  // Knowledge graph, 10-100ms
    Archive,  // Cold storage, 100ms-1s
}

/// Query result with source tier metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResult {
    pub content: serde_json::Value,
    pub tier: MemoryTier,
    pub relevance_score: f32,
    pub timestamp: i64,
    pub metadata: Option<serde_json::Value>,
}

/// Unified memory substrate interface
///
/// Coordinates queries across all four memory tiers in parallel.
pub struct MemorySubstrate {
    working: Arc<WorkingMemory>,
    session: Option<Arc<SessionMemory>>,
    knowledge_graph: Option<Arc<KnowledgeGraphMemory>>,
    archive: Option<Arc<ArchiveMemory>>,
}

impl MemorySubstrate {
    /// Create a new memory substrate
    ///
    /// # Arguments
    /// - `data_dir` - Base directory for persistent storage
    /// - `session_id` - Session identifier
    /// - `project_name` - Project name for knowledge graph and archive
    pub async fn new(
        data_dir: &Path,
        session_id: SessionId,
        project_name: &str,
        llm_client: Option<Arc<dyn LlmClient>>,
    ) -> Result<Self> {
        let working = Arc::new(WorkingMemory::new());

        let session = SessionMemory::new(session_id, data_dir)
            .context("Failed to initialize session memory")?;

        let knowledge_graph = KnowledgeGraphMemory::new(data_dir, project_name, llm_client)
            .await
            .context("Failed to initialize knowledge graph memory")?;

        let archive = ArchiveMemory::new(data_dir, project_name)
            .context("Failed to initialize archive memory")?;

        Ok(Self {
            working,
            session: Some(Arc::new(session)),
            knowledge_graph: Some(Arc::new(knowledge_graph)),
            archive: Some(Arc::new(archive)),
        })
    }

    /// Create a minimal memory substrate with only working memory
    pub fn working_only() -> Self {
        Self {
            working: Arc::new(WorkingMemory::new()),
            session: None,
            knowledge_graph: None,
            archive: None,
        }
    }

    /// Query across all memory tiers in parallel
    ///
    /// # Arguments
    /// - `query_text` - Search query string
    /// - `max_results` - Maximum results per tier
    ///
    /// # Returns
    /// Merged results from all tiers, sorted by relevance
    pub async fn query(
        &self,
        query_text: &str,
        max_results: usize,
    ) -> Result<Vec<MemoryResult>> {
        let mut all_results = Vec::new();

        // Query Tier 1: Working Memory
        if let Some(value) = self.working.get(query_text) {
            all_results.push(MemoryResult {
                content: value,
                tier: MemoryTier::Working,
                relevance_score: 1.0, // Exact match
                timestamp: chrono::Utc::now().timestamp(),
                metadata: None,
            });
        }

        // Query Tier 2: Session Memory (if available)
        if let Some(session) = &self.session {
            let session_results = session
                .search(query_text, max_results)
                .context("Session search failed")?;

            for entry in session_results {
                all_results.push(MemoryResult {
                    content: entry.content,
                    tier: MemoryTier::Session,
                    relevance_score: 0.8,
                    timestamp: entry.timestamp,
                    metadata: entry.metadata,
                });
            }
        }

        // Query Tier 3: Knowledge Graph (if available)
        if let Some(kg) = &self.knowledge_graph {
            let kg_results = kg
                .search(SearchQuery::Semantic {
                    query_text: query_text.to_string(),
                    top_k: max_results,
                    node_types: None,
                })
                .await
                .context("Knowledge graph search failed")?;

            for result in kg_results {
                all_results.push(MemoryResult {
                    content: serde_json::from_str(&result.node.content).unwrap_or(serde_json::Value::Null),
                    tier: MemoryTier::Project,
                    relevance_score: result.relevance_score * 0.7,
                    timestamp: result.node.created_at,
                    metadata: Some(serde_json::from_str(&result.node.metadata).unwrap_or(serde_json::Value::Null)),
                });
            }
        }

        // Query Tier 4: Archive (if available)
        if let Some(archive) = &self.archive {
            let archive_query_result = archive
                .query(ArchiveQuery::Search {
                    query_text: query_text.to_string(),
                    start_timestamp: None,
                    end_timestamp: None,
                    limit: max_results,
                })
                .context("Archive search failed")?;

            for entry in archive_query_result.entries {
                all_results.push(MemoryResult {
                    content: entry.content,
                    tier: MemoryTier::Archive,
                    relevance_score: 0.5,
                    timestamp: entry.timestamp,
                    metadata: entry.metadata,
                });
            }
        }

        // Apply Recency & Relevance Scorer (Time-decay)
        let now = chrono::Utc::now().timestamp();
        for result in &mut all_results {
            let age_seconds = (now - result.timestamp).max(0) as f32;
            // Decay factor: half-life of 24 hours (86400 seconds)
            let decay = (-age_seconds / 86400.0).exp();
            result.relevance_score *= decay;
        }

        // Sort by adjusted relevance score (descending)
        all_results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit total results
        all_results.truncate(max_results * 4);

        Ok(all_results)
    }

    /// Store data in working memory
    pub fn store_working(&self, key: String, value: serde_json::Value) {
        let _ = self.working.insert(key, value, 1.0, false);
    }

    /// Get reference to working memory
    pub fn working(&self) -> &WorkingMemory {
        &self.working
    }

    /// Get reference to session memory
    pub fn session(&self) -> Option<&SessionMemory> {
        self.session.as_deref()
    }

    /// Get reference to knowledge graph memory
    pub fn knowledge_graph(&self) -> Option<&KnowledgeGraphMemory> {
        self.knowledge_graph.as_deref()
    }

    /// Get reference to archive memory
    pub fn archive(&self) -> Option<&ArchiveMemory> {
        self.archive.as_deref()
    }
}

impl Default for MemorySubstrate {
    fn default() -> Self {
        Self::working_only()
    }
}
