use serde::{Deserialize, Serialize};
use zed42_llm::ModelConfig;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// Defines the intelligence ladder for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProfile {
    pub agent_id: String,
    pub tier_1: ModelConfig,
    pub tier_2: Option<ModelConfig>,
    pub tier_3: Option<ModelConfig>,
}

impl ExecutionProfile {
    pub fn new(agent_id: &str, tier_1: ModelConfig) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            tier_1,
            tier_2: None,
            tier_3: None,
        }
    }

    pub fn with_tier_2(mut self, config: ModelConfig) -> Self {
        self.tier_2 = Some(config);
        self
    }

    pub fn with_tier_3(mut self, config: ModelConfig) -> Self {
        self.tier_3 = Some(config);
        self
    }
}

/// Log of a routing decision and execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingLog {
    #[serde(skip)]
    pub id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub original_prompt_len: usize,
    pub selected_tier: u8,
    pub selected_model: String,
    pub retry_count: u8,
    pub failover_reason: Option<String>,
    pub cost: Option<Decimal>,
    pub is_critical: bool,
}
