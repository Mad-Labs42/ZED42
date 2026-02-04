//! ZED42 LLM Integration - OpenRouter Client
//!
//! Copyright (c) 2025 ZED42 Team. All rights reserved.
//! This software is proprietary. See LICENSE file for terms.
//!
//! LLM client for development phase using OpenRouter.
//! Will be replaced with local models in production.

mod client;
mod constrained;
mod prompts;
mod schema;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use client::{LlmClient, MockLlmClient, OpenRouterClient};
pub use constrained::{ConstrainedGen, ConstrainedGenConfig};
pub use prompts::{PromptTemplate, PromptVariable};
pub use schema::{JsonSchema, SchemaBuilder};
pub use types::{LlmError, LlmRequest, LlmResponse, EmbeddingRequest, EmbeddingResponse, ModelConfig, StreamChunk, Result, RetryCause, Usage};

