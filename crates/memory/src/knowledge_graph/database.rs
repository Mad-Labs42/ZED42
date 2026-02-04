//! Core knowledge graph database operations

use super::types::{EdgeType, GraphStats, KnowledgeEdge, KnowledgeNode, NodeType};
use anyhow::{Context, Result};
use std::path::Path;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;
use std::sync::Arc;
use zed42_llm::LlmClient;

/// Knowledge Graph Memory - Tier 3
pub struct KnowledgeGraphMemory {
    pub(crate) db: Surreal<Db>,
    pub(crate) db_path: std::path::PathBuf,
    pub(crate) llm_client: Option<Arc<dyn LlmClient>>,
}

impl KnowledgeGraphMemory {
    /// Initialize Knowledge Graph memory
    ///
    /// # Arguments
    /// - `data_dir` - Directory for storage
    /// - `db_name` - Name of the database/namespace
    /// - `llm_client` - Optional LLM client for semantic operations
    pub async fn new(data_dir: &Path, db_name: &str, llm_client: Option<Arc<dyn LlmClient>>) -> Result<Self> {
        std::fs::create_dir_all(data_dir)
            .context("Failed to create knowledge graph data directory")?;

        let db_path = data_dir.join(format!("{}.db", db_name));

        let db: Surreal<Db> = Surreal::new::<RocksDb>(db_path.clone())
            .await
            .context("Failed to initialize SurrealDB")?;

        let _: () = db.use_ns("zed42")
            .use_db(db_name)
            .await
            .context("Failed to select namespace/database")?;

        let memory = Self { db, db_path, llm_client };
        memory.initialize_schema().await?;
        Ok(memory)
    }

    pub(crate) async fn initialize_schema(&self) -> Result<()> {
        self.db.query("
            DEFINE TABLE IF NOT EXISTS nodes SCHEMALESS;
            DEFINE TABLE IF NOT EXISTS edges SCHEMALESS;
        ").await?;
        Ok(())
    }

    /// Insert a node into the graph
    pub async fn insert_node(&self, node: KnowledgeNode) -> Result<()> {
        self.db.query("CREATE nodes CONTENT $node")
            .bind(("node", node))
            .await
            .context("Failed to insert node")?;
        Ok(())
    }

    /// Insert an edge into the graph
    pub async fn insert_edge(&self, edge: KnowledgeEdge) -> Result<()> {
        self.db.query("CREATE edges CONTENT $edge")
            .bind(("edge", edge))
            .await
            .context("Failed to insert edge")?;
        Ok(())
    }

    /// Retrieve a node by ID
    pub async fn get_node(&self, id: &str) -> Result<Option<KnowledgeNode>> {
        // NATIVE COERCION: Coerce RecordID to String server-side to bypass driver deserialization conflicts
        let mut resp = self.db.query("SELECT *, meta::id(id) AS id FROM nodes WHERE id = $id OR id = type::thing('nodes', $id)")
            .bind(("id", id.to_string()))
            .await?;
        let node: Option<KnowledgeNode> = resp.take(0)?;
        Ok(node)
    }

    /// Retrieve nodes by type
    pub async fn get_nodes_by_type(&self, node_type: NodeType) -> Result<Vec<KnowledgeNode>> {
        let mut response = self.db.query("SELECT *, meta::id(id) AS id FROM nodes WHERE node_type = $type")
            .bind(("type", node_type))
            .await?;
        let nodes: Vec<KnowledgeNode> = response.take(0)?;
        Ok(nodes)
    }

    /// Retrieve edges by type
    pub async fn get_edges_by_type(&self, edge_type: EdgeType) -> Result<Vec<KnowledgeEdge>> {
        let mut response = self.db.query("SELECT *, meta::id(id) AS id FROM edges WHERE edge_type = $type")
            .bind(("type", edge_type))
            .await?;
        let edges: Vec<KnowledgeEdge> = response.take(0)?;
        Ok(edges)
    }

    /// Delete a node and its incident edges
    pub async fn delete_node(&self, id: &str) -> Result<()> {
        let id_owned = id.to_string();
        // TRANSACTIONAL CASCADE DELETE: Atomic removal of node and connected edges
        self.db.query("
            BEGIN TRANSACTION;
            DELETE nodes WHERE id = $id OR id = type::thing('nodes', $id);
            DELETE edges WHERE 
                from_id = $id OR from_id = type::thing('nodes', $id) OR
                to_id = $id OR to_id = type::thing('nodes', $id);
            COMMIT TRANSACTION;
        ")
            .bind(("id", id_owned))
            .await
            .context("Failed to cascade delete node")?;
        Ok(())
    }

    /// Get graph statistics
    pub async fn stats(&self) -> Result<GraphStats> {
        let mut node_resp = self.db.query("SELECT count() FROM nodes GROUP ALL").await?;
        let node_count: Option<serde_json::Value> = node_resp.take(0)?;
        let n = node_count.and_then(|v| v.get("count").and_then(|c| c.as_i64())).unwrap_or(0);

        let mut edge_resp = self.db.query("SELECT count() FROM edges GROUP ALL").await?;
        let edge_count: Option<serde_json::Value> = edge_resp.take(0)?;
        let e = edge_count.and_then(|v| v.get("count").and_then(|c| c.as_i64())).unwrap_or(0);

        Ok(GraphStats {
            node_count: n as usize,
            edge_count: e as usize,
            db_path: self.db_path.clone(),
        })
    }
}
