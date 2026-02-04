use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use rust_decimal_macros::dec;
use surrealdb::engine::any::{connect, Any};
use surrealdb::Surreal;
use zed42_ledger::{IntelligenceLedger, types::{Budget, BudgetStatus}};
use zed42_llm::{LlmClient, LlmRequest, LlmResponse, LlmError, ModelConfig, RetryCause, Usage, EmbeddingRequest, EmbeddingResponse};
use zed42_mom::{Router, types::ExecutionProfile};

#[derive(Clone)]
struct TrackingClient {
    name: String,
    calls: Arc<Mutex<Vec<String>>>, 
    responses: Arc<Mutex<Vec<Result<LlmResponse, LlmError>>>>,
}

impl TrackingClient {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            calls: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn push_response(&self, result: Result<LlmResponse, LlmError>) {
        self.responses.lock().unwrap().push(result);
    }
    
    // Simple FIFO pop. Note: Vectors push to back. remove(0) is front.
    fn pop_response(&self) -> Result<LlmResponse, LlmError> {
        let mut res = self.responses.lock().unwrap();
        if res.is_empty() {
             return Err(LlmError::ApiError("Mock: No more responses".to_string()));
        }
        res.remove(0)
    }
}

#[async_trait]
impl LlmClient for TrackingClient {
    async fn complete(&self, request: LlmRequest) -> zed42_llm::Result<LlmResponse> {
        self.calls.lock().unwrap().push(format!("{}: {}", self.name, request.prompt));
        self.pop_response()
    }

    async fn stream(&self, _request: LlmRequest) -> zed42_llm::Result<Vec<zed42_llm::StreamChunk>> {
        unimplemented!()
    }

    async fn embed(&self, _request: EmbeddingRequest) -> zed42_llm::Result<EmbeddingResponse> {
        Ok(EmbeddingResponse {
            embedding: vec![0.0; 1536],
            model: "mock-embed".to_string(),
            usage: Usage::default(),
        })
    }
}

async fn setup_env() -> (Router, Arc<IntelligenceLedger>, Surreal<Any>) {
    let db = connect("mem://").await.unwrap();
    db.use_ns("zed42").use_db("mom").await.unwrap();
    
    let ledger = Arc::new(IntelligenceLedger::new(db.clone()));
    let default_client = Arc::new(TrackingClient::new("default"));
    let router = Router::new(ledger.clone(), db.clone(), default_client);
    
    // Set Infinite Budget
    ledger.set_budget(Budget {
        entity_id: "default".to_string(),
        hard_limit: dec!(1000.0),
        soft_limit: dec!(500.0),
        spent: dec!(0.0),
        currency: "USD".to_string(),
        updated_at: chrono::Utc::now(),
        status: BudgetStatus::Active,
    }).await.unwrap();

    // Register cleanup model rate to avoid settlement failures during LeaseGuard drop
    ledger.set_rate(zed42_ledger::types::RateTableEntry {
        model: "lease-guard-cleanup".to_string(),
        input_cost_per_1k: dec!(0.0),
        output_cost_per_1k: dec!(0.0),
    }).await.unwrap();

    (router, ledger, db)
}

#[tokio::test]
async fn test_waterfall_failover() {
    let (mut router, _, db) = setup_env().await;
    
    // Setup Clients
    let tier1_client = Arc::new(TrackingClient::new("tier1")); 
    let tier2_client = Arc::new(TrackingClient::new("tier2")); 
    
    // Tier 1 fails 3 times (exhausting retries)
    // MOM max_retries = 2 (so 3 attempts total)
    tier1_client.push_response(Err(LlmError::RateLimitExceeded));
    tier1_client.push_response(Err(LlmError::RateLimitExceeded));
    tier1_client.push_response(Err(LlmError::RateLimitExceeded));

    // Tier 2 succeeds
    tier2_client.push_response(Ok(LlmResponse {
        content: "Success".to_string(),
        model: "tier2-model".to_string(),
        usage: Usage { prompt_tokens: 10, completion_tokens: 10, total_tokens: 20 },
        finish_reason: "stop".to_string(),
    }));

    router.register_client("tier1", tier1_client.clone());
    router.register_client("tier2", tier2_client.clone());

    // Inject Profile
    let tier1_config = ModelConfig { model: "tier1-model".to_string(), ..ModelConfig::default() };
    let tier2_config = ModelConfig { model: "tier2-model".to_string(), ..ModelConfig::default() };
    
    let profile = ExecutionProfile::new("default", tier1_config)
        .with_tier_2(tier2_config);
    
    let _: Option<ExecutionProfile> = db.create(("model_profiles", "default"))
        .content(profile).await.unwrap();

    // Execute
    let request = LlmRequest::new("Hello".to_string()).agent("default".to_string());
    let response = router.complete(request).await.expect("Router failed");

    assert_eq!(response.model, "tier2-model");
    
    // Verify Calls
    let t1_calls = tier1_client.calls.lock().unwrap();
    assert_eq!(t1_calls.len(), 3, "Tier 1 should be called 3 times (1 initial + 2 retries)");
    
    let t2_calls = tier2_client.calls.lock().unwrap();
    assert_eq!(t2_calls.len(), 1, "Tier 2 should be called once");
}

#[tokio::test]
async fn test_smart_escalation() {
    let (mut router, _, db) = setup_env().await;
    
    // Setup Clients
    let tier1_client = Arc::new(TrackingClient::new("tier1")); 
    let tier2_client = Arc::new(TrackingClient::new("tier2")); 
    
    // Tier 2 succeeds (Tier 1 should NOT be called)
    tier2_client.push_response(Ok(LlmResponse {
        content: "Smart Success".to_string(),
        model: "tier2-model".to_string(),
        usage: Usage { prompt_tokens: 10, completion_tokens: 10, total_tokens: 20 },
        finish_reason: "stop".to_string(),
    }));

    router.register_client("tier1", tier1_client.clone());
    router.register_client("tier2", tier2_client.clone());

    // Inject Profile
    let tier1_config = ModelConfig { model: "tier1-model".to_string(), ..ModelConfig::default() };
    let tier2_config = ModelConfig { model: "tier2-model".to_string(), ..ModelConfig::default() };
    
    let profile = ExecutionProfile::new("default", tier1_config)
        .with_tier_2(tier2_config);
    
    let _: Option<ExecutionProfile> = db.create(("model_profiles", "default"))
        .content(profile).await.unwrap();

    // Execute with retry_cause = ValidationFailure
    let request = LlmRequest::new("Hello".to_string())
        .agent("default".to_string())
        .retry_cause(RetryCause::ValidationFailure);
        
    let response = router.complete(request).await.expect("Router failed");

    assert_eq!(response.model, "tier2-model");
    
    // Verify Calls
    let t1_calls = tier1_client.calls.lock().unwrap();
    assert_eq!(t1_calls.len(), 0, "Tier 1 should be skipped due to ValidationFailure");
    
    let t2_calls = tier2_client.calls.lock().unwrap();
    assert_eq!(t2_calls.len(), 1, "Tier 2 should be called once");
}

#[tokio::test]
async fn test_circuit_breaker_transparency() {
    let (mut router, _, db) = setup_env().await;
    
    let tier1_client = Arc::new(TrackingClient::new("tier1")); 
    // Fail 3 times to open circuit
    // Note: MOM loop handles retries. If we return a non-retryable error, it breaks immediately.
    // If we return a retryable error, it retries up to 2 times (3 total).
    tier1_client.push_response(Err(LlmError::RateLimitExceeded));
    tier1_client.push_response(Err(LlmError::RateLimitExceeded));
    tier1_client.push_response(Err(LlmError::RateLimitExceeded));

    router.register_client("tier1", tier1_client.clone());
    
    let tier1_config = ModelConfig { model: "tier1-model".to_string(), ..ModelConfig::default() };
    let profile = ExecutionProfile::new("default", tier1_config);
    let _: Option<ExecutionProfile> = db.create(("model_profiles", "default"))
        .content(profile).await.unwrap();

    let request = LlmRequest::new("Hello".to_string()).agent("default".to_string());
    
    // Call 3 times to exhaust 3-strike threshold (MOM retries don't count individually yet)
    let _ = router.complete(request.clone()).await;
    let _ = router.complete(request.clone()).await;
    let _ = router.complete(request.clone()).await;

    let status = router.get_circuit_status();
    let t1_status = status.iter().find(|s| s.model == "tier1-model").expect("Model status missing");
    assert!(t1_status.is_open, "Circuit should be OPEN after 3 failures");
    assert_eq!(t1_status.failures, 3);
}

#[tokio::test]
async fn test_lease_cleanup_on_early_return() {
    let (mut router, _, db) = setup_env().await;
    
    let tier1_client = Arc::new(TrackingClient::new("tier1")); 
    // Non-retryable error should trigger break and cleanup
    tier1_client.push_response(Err(LlmError::InvalidResponse("Garbage".to_string())));

    router.register_client("tier1", tier1_client.clone());
    let tier1_config = ModelConfig { model: "tier1-model".to_string(), ..ModelConfig::default() };
    let profile = ExecutionProfile::new("default", tier1_config);
    let _: Option<ExecutionProfile> = db.create(("model_profiles", "default"))
        .content(profile).await.unwrap();

    let request = LlmRequest::new("Hello".to_string()).agent("default".to_string());
    let _ = router.complete(request).await;

    // Give more time for LeaseGuard spawn to run
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Check if any leases are left
    let leases: Vec<serde_json::Value> = db.select("leases").await.unwrap();
    assert!(leases.is_empty(), "Lease should have been cleaned up by LeaseGuard. Found: {:?}", leases);
}

#[tokio::test]
async fn test_canary_recovery() {
    let (mut router, _, db) = setup_env().await;
    
    // Set short reset timeout for testing
    let cb = zed42_mom::circuit_breaker::CircuitBreaker::new()
        .with_thresholds(1, std::time::Duration::from_millis(200), std::time::Duration::from_secs(1));
    router = router.with_circuit_breaker(cb);

    let tier1_client = Arc::new(TrackingClient::new("tier1")); 
    tier1_client.push_response(Err(LlmError::ApiError("FAIL".to_string()))); // Trip 1
    
    router.register_client("tier1", tier1_client.clone());
    let tier1_config = ModelConfig { model: "tier1-model".to_string(), ..ModelConfig::default() };
    let profile = ExecutionProfile::new("default", tier1_config.clone());
    let _: Option<ExecutionProfile> = db.create(("model_profiles", "default"))
        .content(profile).await.unwrap();

    let request = LlmRequest::new("Hello".to_string()).agent("default".to_string());
    
    // 1. Trip the circuit
    let _ = router.complete(request.clone()).await;
    {
        let status = router.get_circuit_status();
        let t1 = status.iter().find(|s| s.model == "tier1-model").unwrap();
        assert!(t1.is_open);
        assert_eq!(t1.state, "Open");
    }

    // 2. Wait for timeout and send canary
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    
    // Success on canary
    tier1_client.push_response(Ok(LlmResponse {
        content: "Canary Success".to_string(),
        model: "tier1-model".to_string(),
        usage: Usage { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 },
        finish_reason: "stop".to_string(),
    }));

    let resp = router.complete(request.clone()).await.unwrap();
    assert_eq!(resp.content, "Canary Success");

    // 3. Verify circuit CLOSED
    {
        let status = router.get_circuit_status();
        let t1 = status.iter().find(|s| s.model == "tier1-model").unwrap();
        assert!(!t1.is_open);
        assert_eq!(t1.state, "Closed");
    }
}

#[tokio::test]
async fn test_backpressure_signaling() {
    let (mut router, _, db) = setup_env().await;
    
    // Set 1-strike failure threshold
    let cb = zed42_mom::circuit_breaker::CircuitBreaker::new()
        .with_thresholds(1, std::time::Duration::from_secs(300), std::time::Duration::from_secs(30));
    router = router.with_circuit_breaker(cb);

    // Register 5 models
    for i in 1..=5 {
        let name = format!("m{}", i);
        let client = Arc::new(TrackingClient::new(&name));
        // Use 2 fail responses per model just in case of retries
        client.push_response(Err(LlmError::ApiError("FAIL".to_string())));
        client.push_response(Err(LlmError::ApiError("FAIL".to_string())));
        router.register_client(&name, client);
    }

    // Fail 4 models to reach 4/4 = 100% open (which is >= 80%)
    for i in 1..=4 {
        let name = format!("m{}", i);
        let config = ModelConfig { model: name.clone(), ..ModelConfig::default() };
        let profile = ExecutionProfile::new("default", config);
        let _: Option<ExecutionProfile> = db.update(("model_profiles", "default"))
            .content(profile).await.unwrap();
        
        let request = LlmRequest::new("Hello".to_string()).agent("default".to_string());
        let _ = router.complete(request).await;
    }

    // Now check backpressure on the next request
    // Even if we target m5, at the START of the call, 4/4 models in CB are open
    let config5 = ModelConfig { model: "m5".to_string(), ..ModelConfig::default() };
    let profile = ExecutionProfile::new("default", config5);
    let _: Option<ExecutionProfile> = db.update(("model_profiles", "default"))
        .content(profile).await.unwrap();

    let request = LlmRequest::new("Hello".to_string()).agent("default".to_string());
    let err = router.complete(request).await.unwrap_err();
    
    match err {
        LlmError::Backpressure(duration) => {
            assert_eq!(duration, std::time::Duration::from_secs(30));
        }
        _ => panic!("Expected Backpressure error, got {:?}", err),
    }
}
