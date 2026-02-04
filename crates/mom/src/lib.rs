//! Model Orchestration Matrix (MOM)
//!
//! Central routing intelligence for ZED42 agents.

pub mod circuit_breaker;
pub mod types;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::circuit_breaker::CircuitBreaker;
use crate::types::{ExecutionProfile, RoutingLog};
use zed42_ledger::{IntelligenceLedger, types::Usage};
use zed42_llm::LlmClient;
use zed42_llm::{LlmError, LlmRequest, LlmResponse, RetryCause, StreamChunk, EmbeddingRequest, EmbeddingResponse};

/// Guard that ensures a lease is settled or released back to the budget
struct LeaseGuard {
    lease_id: Option<String>,
    ledger: Arc<IntelligenceLedger>,
    settled: bool,
}

impl LeaseGuard {
    fn new(lease_id: String, ledger: Arc<IntelligenceLedger>) -> Self {
        Self {
            lease_id: Some(lease_id),
            ledger,
            settled: false,
        }
    }

    /// Mark the lease as settled and return the lease_id
    fn settle(&mut self) -> String {
        self.settled = true;
        self.lease_id.take().unwrap_or_default()
    }
}

impl Drop for LeaseGuard {
    fn drop(&mut self) {
        if !self.settled {
            if let Some(lease_id) = self.lease_id.take() {
                let ledger = Arc::clone(&self.ledger);
                warn!(lease_id = %lease_id, "Lease leaked! Releasing budget via LeaseGuard");
                tokio::spawn(async move {
                    let _ = ledger.commit_usage(&lease_id, Usage {
                        input_tokens: 0,
                        output_tokens: 0,
                        model: "lease-guard-cleanup".to_string(),
                    }).await;
                });
            }
        }
    }
}

/// The Orchestrator
pub struct Router {
    ledger: Arc<IntelligenceLedger>,
    db: Surreal<Any>,
    circuit_breaker: CircuitBreaker,
    /// Map of provider prefix (e.g., "openai") to client
    clients: HashMap<String, Arc<dyn LlmClient>>,
    /// Fallback client if no specific provider matches
    default_client: Arc<dyn LlmClient>,
}

impl Router {
    pub fn new(
        ledger: Arc<IntelligenceLedger>,
        db: Surreal<Any>,
        default_client: Arc<dyn LlmClient>,
    ) -> Self {
        Self {
            ledger,
            db,
            circuit_breaker: CircuitBreaker::new(),
            clients: HashMap::new(),
            default_client,
        }
    }

    pub fn with_circuit_breaker(mut self, cb: CircuitBreaker) -> Self {
        self.circuit_breaker = cb;
        self
    }

    pub fn register_client(&mut self, prefix: &str, client: Arc<dyn LlmClient>) {
        self.clients.insert(prefix.to_string(), client);
    }

    async fn get_client(&self, model: &str) -> &Arc<dyn LlmClient> {
        for (prefix, client) in &self.clients {
            if model.starts_with(prefix) {
                return client;
            }
        }
        &self.default_client
    }

    async fn get_profile(&self, agent_id: &str) -> anyhow::Result<ExecutionProfile> {
        let profile: Option<ExecutionProfile> = self.db
            .select(("model_profiles", agent_id))
            .await?;
        
        profile.ok_or_else(|| anyhow::anyhow!("No execution profile for agent {}", agent_id))
    }

    async fn log_routing(&self, log: RoutingLog) {
        // Updated to use Option return type and pass owned log
        let _: Option<RoutingLog> = self.db.create("routing_logs").content(log).await.ok().flatten();
    }

    /// Get transparency on model health
    pub fn get_circuit_status(&self) -> Vec<crate::circuit_breaker::CircuitStatus> {
        self.circuit_breaker.get_status()
    }
}

#[async_trait]
impl LlmClient for Router {
    async fn complete(&self, request: LlmRequest) -> zed42_llm::Result<LlmResponse> {
        // 1. Identify Agent
        let agent_id = request.agent_id.as_deref().unwrap_or("default");

        // 2. Check Backpressure
        let total = self.circuit_breaker.total_models();
        if total >= 3 { 
            let open = self.circuit_breaker.count_open();
            if (open as f32 / total as f32) >= 0.8 {
                let wait = Duration::from_secs(30);
                warn!(open = %open, total = %total, "BACKPRESSURE triggered: 80% circuits open");
                
                self.log_routing(RoutingLog {
                    id: None,
                    timestamp: Utc::now(),
                    agent_id: agent_id.to_string(),
                    original_prompt_len: request.prompt.len(),
                    selected_tier: 0,
                    selected_model: "BACKPRESSURE".to_string(),
                    retry_count: 0,
                    failover_reason: Some(format!("ProviderExhaustion: {}/{} circuits open", open, total)),
                    cost: None,
                    is_critical: true,
                }).await;

                return Err(LlmError::Backpressure(wait));
            }
        }

        // 2. Resolve Profile
        let profile = self.get_profile(agent_id).await.unwrap_or_else(|_| {
            ExecutionProfile::new(
                "default",
                request.config.clone(),
            )
        });

        // 3. Determine Starting Tier
        let start_tier = if let Some(RetryCause::ValidationFailure) = request.retry_cause {
            info!("Smart Escalation: Validation failed, skipping to Tier 2");
            2
        } else {
            1
        };

        // 4. Waterfall Loop
        let tiers = [
            (1u8, Some(profile.tier_1)),
            (2u8, profile.tier_2),
            (3u8, profile.tier_3),
        ];

        let mut last_error = LlmError::InvalidResponse("No models configured".to_string());

        for (tier_num, config_opt) in tiers.iter() {
            if *tier_num < start_tier { continue; }
            let config = match config_opt {
                Some(c) => c,
                None => continue,
            };

            // Check Circuit Breaker
            if self.circuit_breaker.is_open(&config.model) {
                warn!(model = %config.model, "Circuit open, skipping tier {}", tier_num);
                continue;
            }

            // Client Selection
            let client = self.get_client(&config.model).await;

            // Financial Handshake
            // Using strict Decimal for estimation
            let est_cost = match *tier_num {
                1 => Decimal::new(1, 2), // $0.01
                2 => Decimal::new(5, 2), // $0.05
                _ => Decimal::new(20, 2), // $0.20
            };

            let lease_id = match self.ledger.request_lease(agent_id, est_cost).await {
                Ok(id) => id,
                Err(e) => {
                    error!(agent = %agent_id, error = %e, "Budget denied");
                    return Err(LlmError::ApiError(format!("Budget exceeded: {}", e)));
                }
            };

            let mut lease_guard = LeaseGuard::new(lease_id, Arc::clone(&self.ledger));

            // Execute with Retries (Backoff)
            let mut attempt = 0;
            let max_retries = 2;
            
            loop {
                let mut req_clone = request.clone();
                req_clone.config = config.clone();

                match client.complete(req_clone).await {
                    Ok(mut response) => {
                        self.circuit_breaker.report_success(&config.model);
                        
                        // Settle Ledger
                        let usage = Usage {
                            input_tokens: response.usage.prompt_tokens as u32,
                            output_tokens: response.usage.completion_tokens as u32,
                            model: config.model.clone(),
                        };
                        
                        let actual_lease_id = lease_guard.settle();
                        let receipt = self.ledger.commit_usage(&actual_lease_id, usage).await;
                        
                        // Log
                        self.log_routing(RoutingLog {
                            id: None,
                            timestamp: Utc::now(),
                            agent_id: agent_id.to_string(),
                            original_prompt_len: request.prompt.len(),
                            selected_tier: *tier_num,
                            selected_model: config.model.clone(),
                            retry_count: attempt,
                            failover_reason: None,
                            cost: receipt.ok().map(|r| r.cost),
                            is_critical: false,
                        }).await;

                        response.model = config.model.clone();
                        return Ok(response);
                    },
                    Err(e) => {
                        attempt += 1;
                        match e {
                            LlmError::RateLimitExceeded | LlmError::NetworkError(_) | LlmError::ApiError(_) => {
                                if attempt <= max_retries {
                                    let wait = Duration::from_millis(100 * (2u64.pow(attempt as u32)));
                                    sleep(wait).await;
                                    continue;
                                }
                            }
                            _ => { break; }
                        }
                        
                        self.circuit_breaker.report_failure(&config.model);
                        last_error = e;
                        break;
                    }
                }
            }
        }

        // Critical Failure: All tiers failed
        error!(agent = %agent_id, "CRITICAL FAILURE: Model loop exhausted all tiers");
        self.log_routing(RoutingLog {
            id: None,
            timestamp: Utc::now(),
            agent_id: agent_id.to_string(),
            original_prompt_len: request.prompt.len(),
            selected_tier: 0,
            selected_model: "NONE".to_string(),
            retry_count: 0,
            failover_reason: Some(format!("All tiers failed: {:?}", last_error)),
            cost: None,
            is_critical: true,
        }).await;

        Err(last_error)
    }

    async fn stream(&self, _request: LlmRequest) -> zed42_llm::Result<Vec<StreamChunk>> {
        Err(LlmError::ApiError("Streaming not yet implemented in Router".to_string()))
    }

    async fn embed(&self, request: EmbeddingRequest) -> zed42_llm::Result<EmbeddingResponse> {
        // Forward to default client or specific embedding client
        // For MVP, just route to default
        self.default_client.embed(request).await
    }
}
