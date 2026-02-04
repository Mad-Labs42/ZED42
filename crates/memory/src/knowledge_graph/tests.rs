use super::*;
use serde_json::json;
use tempfile::TempDir;
use uuid::Uuid;

async fn create_test_graph() -> (KnowledgeGraphMemory, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let graph = KnowledgeGraphMemory::new(temp_dir.path(), "test_kg", None)
        .await
        .unwrap();
    (graph, temp_dir)
}

#[tokio::test]
async fn test_insert_and_get_node() {
    let (graph, _temp) = create_test_graph().await;
    let node_id = Uuid::new_v4().to_string();

    let node = KnowledgeNode {
        id: node_id.clone(),
        node_type: "function".to_string(),
        name: "test_function".to_string(),
        content: json!({"code": "fn test() {}"}).to_string(),
        embedding: None,
        metadata: json!({}).to_string(),
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    graph.insert_node(node).await.unwrap();

    let retrieved = graph.get_node(&node_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test_function");
}

#[tokio::test]
async fn test_insert_edge() {
    let (graph, _temp) = create_test_graph().await;

    let edge = KnowledgeEdge {
        id: Uuid::new_v4().to_string(),
        edge_type: "calls".to_string(),
        from_id: "node1".to_string(),
        to_id: "node2".to_string(),
        metadata: None,
        created_at: chrono::Utc::now().timestamp(),
    };

    graph.insert_edge(edge).await.unwrap();
}

#[tokio::test]
async fn test_thing_coercion_robustness() {
    let (graph, _temp) = create_test_graph().await;

    // Case 1: Simple Alphanumeric ID
    let simple_id = "simple_test_id";
    let node1 = KnowledgeNode {
        id: simple_id.to_string(),
        node_type: "function".to_string(),
        name: "Simple".to_string(),
        content: "Ref".to_string(),
        embedding: None,
        metadata: "{}".to_string(),
        created_at: 0,
        updated_at: 0,
    };
    graph.insert_node(node1.clone()).await.unwrap();

    let retrieved1 = graph.get_node(simple_id).await.unwrap();
    assert!(retrieved1.is_some(), "Failed to retrieve simple ID node");
    assert_eq!(retrieved1.unwrap().id, simple_id, "ID mismatch for simple ID");

    // Case 2: ID resembling a RecordId (with colon)
    let complex_id = "complex:id:test";
    let node2 = KnowledgeNode {
        id: complex_id.to_string(),
        node_type: "function".to_string(),
        name: "Complex".to_string(),
        content: "Ref".to_string(),
        embedding: None,
        metadata: "{}".to_string(),
        created_at: 0,
        updated_at: 0,
    };
    graph.insert_node(node2.clone()).await.unwrap();

    let retrieved2 = graph.get_node(complex_id).await.unwrap();
    assert!(retrieved2.is_some(), "Failed to retrieve complex ID node");
    assert_eq!(retrieved2.unwrap().id, complex_id, "ID mismatch for complex ID");
    
    // Case 3: Edge deserialization (Thing vs String)
    // We insert an edge. Since we verify "get_edges_by_type", we need to make sure 
    // the helper works when retrieving.
    let edge_id = "edge:complex:id";
    let edge = KnowledgeEdge {
        id: edge_id.to_string(),
        edge_type: "calls".to_string(),
        from_id: simple_id.to_string(),
        to_id: complex_id.to_string(),
        metadata: None,
        created_at: 0,
    };
    graph.insert_edge(edge).await.unwrap();

    let edges = graph.get_edges_by_type(EdgeType::Calls).await.unwrap();
    assert!(!edges.is_empty(), "Failed to retrieve edges");
    // Verify IDs coming back are strings
    let retrieved_edge = edges.iter().find(|e| e.id == edge_id).expect("Edge not found");
    assert_eq!(retrieved_edge.from_id, simple_id, "from_id mismatch");
    assert_eq!(retrieved_edge.to_id, complex_id, "to_id mismatch");
}

#[tokio::test]
async fn test_cascade_delete() {
    let (graph, _temp) = create_test_graph().await;

    let node1_id = "node1";
    let node2_id = "node2";

    // Insert nodes
    let node1 = KnowledgeNode {
        id: node1_id.to_string(),
        node_type: "function".to_string(),
        name: "N1".to_string(),
        content: "".to_string(),
        embedding: None,
        metadata: "{}".to_string(),
        created_at: 0,
        updated_at: 0,
    };
    graph.insert_node(node1).await.unwrap();
    
    let node2 = KnowledgeNode {
        id: node2_id.to_string(),
        node_type: "function".to_string(),
        name: "N2".to_string(),
        content: "".to_string(),
        embedding: None,
        metadata: "{}".to_string(),
        created_at: 0,
        updated_at: 0,
    };
    graph.insert_node(node2).await.unwrap();

    // Insert Edge N1 -> N2
    let edge = KnowledgeEdge {
        id: "edge1".to_string(),
        edge_type: "calls".to_string(),
        from_id: node1_id.to_string(),
        to_id: node2_id.to_string(),
        metadata: None,
        created_at: 0,
    };
    graph.insert_edge(edge).await.unwrap();
    
    // Verify edge exists
    let edges = graph.get_edges_by_type(EdgeType::Calls).await.unwrap();
    assert_eq!(edges.len(), 1);

    // Delete Node 1 (Source)
    graph.delete_node(node1_id).await.unwrap();

    // Verify Node 1 is gone
    assert!(graph.get_node(node1_id).await.unwrap().is_none());
    
    // Verify Edge is gone (Cascade)
    let edges_after = graph.get_edges_by_type(EdgeType::Calls).await.unwrap();
    assert_eq!(edges_after.len(), 0, "Edge should have been cascade deleted");
}

#[tokio::test]
async fn test_semantic_search() {
    let (graph, _temp) = create_test_graph().await;

    // Insert test nodes
    for i in 0..3 {
        let node = KnowledgeNode {
            id: format!("node{}", i),
            node_type: "function".to_string(),
            name: format!("test_function_{}", i),
            content: json!({"code": format!("fn test_{}() {{}}", i)}).to_string(),
            embedding: None,
            metadata: json!({}).to_string(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        graph.insert_node(node).await.unwrap();
    }

    // Skip semantic search if not configured
    if graph.llm_client.is_some() {
        let results = graph
            .search(SearchQuery::Semantic {
                query_text: "test_function".to_string(),
                top_k: 2,
                node_types: None,
            })
            .await
            .unwrap();

        assert!(!results.is_empty());
    }
}

#[tokio::test]
async fn test_delete_node() {
    let (graph, _temp) = create_test_graph().await;

    let node = KnowledgeNode {
        id: "delete_me".to_string(),
        node_type: "function".to_string(),
        name: "to_delete".to_string(),
        content: json!({}).to_string(),
        embedding: None,
        metadata: json!({}).to_string(),
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    graph.insert_node(node).await.unwrap();
    graph.delete_node("delete_me").await.unwrap();

    let retrieved = graph.get_node("delete_me").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_stats() {
    let (graph, _temp) = create_test_graph().await;

    let stats = graph.stats().await.unwrap();
    // Should start with 0 nodes
    assert_eq!(stats.node_count, 0);
}
