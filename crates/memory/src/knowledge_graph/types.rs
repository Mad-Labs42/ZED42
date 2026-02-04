use serde::{Deserialize, Serialize};

/// Knowledge graph node types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// Code file
    File,
    /// Function/method definition
    Function,
    /// Class/struct/type definition
    Type,
    /// Module/package
    Module,
    /// Architectural decision
    Decision,
    /// Dependency relationship
    Dependency,
    /// Test case
    Test,
    /// Documentation
    Documentation,
}

impl From<String> for NodeType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "file" => NodeType::File,
            "function" => NodeType::Function,
            "type" => NodeType::Type,
            "module" => NodeType::Module,
            "decision" => NodeType::Decision,
            "dependency" => NodeType::Dependency,
            "test" => NodeType::Test,
            "documentation" => NodeType::Documentation,
            _ => NodeType::Documentation,
        }
    }
}

/// Edge types for graph relationships
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    /// A calls B
    Calls,
    /// A depends on B
    DependsOn,
    /// A implements B (interface/trait)
    Implements,
    /// A contains B (module contains function)
    Contains,
    /// A tests B
    Tests,
    /// A documents B
    Documents,
    /// A is related to decision B
    ImplementsDecision,
    /// A supersedes B (newer version)
    Supersedes,
}

impl From<String> for EdgeType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "calls" => EdgeType::Calls,
            "depends_on" => EdgeType::DependsOn,
            "implements" => EdgeType::Implements,
            "contains" => EdgeType::Contains,
            "tests" => EdgeType::Tests,
            "documents" => EdgeType::Documents,
            "implements_decision" => EdgeType::ImplementsDecision,
            "supersedes" => EdgeType::Supersedes,
            _ => EdgeType::DependsOn,
        }
    }
}

/// Knowledge graph node (Coerced String-Substrate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Custom deserializer for SurrealDB IDs (Thing vs String)
fn deserialize_id_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IdOrThing {
        String(String),
        Thing { tb: String, id: String },
        ThingObj { tb: String, id: serde_json::Value }, 
    }

    match IdOrThing::deserialize(deserializer)? {
        IdOrThing::String(s) => Ok(s),
        IdOrThing::Thing { tb, id } => Ok(format!("{}:{}", tb, id)),
        IdOrThing::ThingObj { tb, id } => Ok(format!("{}:{}", tb, id)),
    }
}

/// Knowledge graph edge (Coerced String-Substrate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub id: String,
    pub edge_type: String,
    #[serde(deserialize_with = "deserialize_id_to_string")]
    pub from_id: String,
    #[serde(deserialize_with = "deserialize_id_to_string")]
    pub to_id: String,
    pub metadata: Option<String>,
    pub created_at: i64,
}

/// Search result from knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub node: KnowledgeNode,
    pub relevance_score: f32,
    pub path: Option<Vec<String>>,
}

/// Search query for knowledge graph
#[derive(Debug, Clone)]
pub enum SearchQuery {
    /// Vector similarity search
    Semantic {
        query_text: String,
        top_k: usize,
        node_types: Option<Vec<NodeType>>,
    },
    /// Graph traversal search
    Structural {
        start_node_id: String,
        edge_types: Vec<EdgeType>,
        max_depth: usize,
    },
    /// Combined semantic + structural
    Hybrid {
        query_text: String,
        start_node_id: Option<String>,
        top_k: usize,
        max_depth: usize,
    },
    /// Temporal query (state at specific time)
    Temporal {
        target_timestamp: i64,
        node_types: Option<Vec<NodeType>>,
    },
}

/// Knowledge graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub db_path: std::path::PathBuf,
}
