//! Knowledge graph search operations

use super::database::KnowledgeGraphMemory;
use super::types::{EdgeType, KnowledgeNode, NodeType, SearchQuery, SearchResult};
use anyhow::{Result, Context};

impl KnowledgeGraphMemory {
    /// Search the knowledge graph
    ///
    /// # Arguments
    /// - `query` - Search query specification
    ///
    /// # Returns
    /// Vector of search results with relevance scores
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        match query {
            SearchQuery::Semantic {
                query_text,
                top_k,
                node_types,
            } => self.semantic_search(&query_text, top_k, node_types.as_deref()).await,

            SearchQuery::Structural {
                start_node_id,
                edge_types,
                max_depth,
            } => {
                self.structural_search(&start_node_id, &edge_types, max_depth)
                    .await
            }

            SearchQuery::Hybrid {
                query_text,
                start_node_id,
                top_k,
                max_depth,
            } => {
                self.hybrid_search(&query_text, start_node_id.as_deref(), top_k, max_depth)
                    .await
            }

            SearchQuery::Temporal {
                target_timestamp,
                node_types,
            } => self.temporal_search(target_timestamp, node_types.as_deref()).await,
        }
    }

    /// Semantic vector similarity search
    pub(crate) async fn semantic_search(
        &self,
        query_text: &str,
        top_k: usize,
        node_types: Option<&[NodeType]>,
    ) -> Result<Vec<SearchResult>> {
        let client = self.llm_client.as_ref()
            .context("LLM client not configured for semantic search")?;

        // 1. Generate embedding for the query
        let embedding_resp = client.embed(zed42_llm::EmbeddingRequest::new(query_text.to_string())).await?;
        let query_embedding = embedding_resp.embedding;

        // 2. Build SurrealQL vector search query
        let type_filter = if let Some(types) = node_types {
            let type_strs: Vec<String> = types.iter().map(|t| format!("'{:?}'", t)).collect();
            format!("AND node_type IN [{}]", type_strs.join(", "))
        } else {
            String::new()
        };

        // Use native vector search syntax with cosine distance
        // The <40> is a threshold/limit for the MTREE index search
        let query_str = format!(
            "SELECT *, vector::distance::cosine(embedding, $query_vec) AS dist 
             FROM nodes 
             WHERE embedding < 40 > $query_vec {} 
             ORDER BY dist ASC 
             LIMIT {}",
            type_filter, top_k
        );

        let mut response: surrealdb::Response = self.db.query(query_str)
            .bind(("query_vec", query_embedding))
            .await?;
        
        let nodes: Vec<KnowledgeNode> = response.take(0)?;

        let results = nodes
            .into_iter()
            .map(|node| {
                // SurrealDB returns 'dist', we want 'relevance_score' (1 - dist)
                // Since cosine distance is [0, 2] for normalized vectors, or [0, 1] usually
                // and we want 1.0 to be best.
                SearchResult {
                    node,
                    relevance_score: 1.0, // We could extract the dist but KnowledgeNode doesn't have it
                    path: None,
                }
            })
            .collect();

        Ok(results)
    }

    /// Structural graph traversal search
    pub(crate) async fn structural_search(
        &self,
        start_node_id: &str,
        edge_types: &[EdgeType],
        _max_depth: usize,
    ) -> Result<Vec<SearchResult>> {
        // Graph traversal query
        let edge_type_strs: Vec<String> = edge_types.iter().map(|t| format!("{:?}", t)).collect();

        let query_str = format!(
            "SELECT * FROM nodes WHERE id IN (
                SELECT to_id FROM edges
                WHERE from_id = '{}' AND edge_type IN [{}]
            ) LIMIT 100",
            start_node_id,
            edge_type_strs.join(", ")
        );

        let mut response: surrealdb::Response = self.db.query(query_str).await?;
        let nodes: Vec<KnowledgeNode> = response.take(0)?;

        let results = nodes
            .into_iter()
            .map(|node| SearchResult {
                node,
                relevance_score: 1.0,
                path: Some(vec![start_node_id.to_string()]),
            })
            .collect();

        Ok(results)
    }

    /// Hybrid semantic + structural search
    pub(crate) async fn hybrid_search(
        &self,
        query_text: &str,
        start_node_id: Option<&str>,
        top_k: usize,
        max_depth: usize,
    ) -> Result<Vec<SearchResult>> {
        // Combine semantic and structural results
        let semantic_results = self.semantic_search(query_text, top_k, None).await?;

        if let Some(start_id) = start_node_id {
            // TODO: Combine with structural search and re-rank
            self.structural_search(start_id, &[], max_depth).await
        } else {
            Ok(semantic_results)
        }
    }

    /// Temporal query - get state at specific timestamp
    pub(crate) async fn temporal_search(
        &self,
        target_timestamp: i64,
        node_types: Option<&[NodeType]>,
    ) -> Result<Vec<SearchResult>> {
        let type_filter = if let Some(types) = node_types {
            let type_strs: Vec<String> = types.iter().map(|t| format!("{:?}", t)).collect();
            format!("AND node_type IN [{}]", type_strs.join(", "))
        } else {
            String::new()
        };

        let query_str = format!(
            "SELECT * FROM nodes
             WHERE created_at <= {} AND (updated_at >= {} OR updated_at IS NONE) {}
             LIMIT 1000",
            target_timestamp, target_timestamp, type_filter
        );

        let mut response: surrealdb::Response = self.db.query(query_str).await?;
        let nodes: Vec<KnowledgeNode> = response.take(0)?;

        let results = nodes
            .into_iter()
            .map(|node| SearchResult {
                node,
                relevance_score: 1.0,
                path: None,
            })
            .collect();

        Ok(results)
    }
}
