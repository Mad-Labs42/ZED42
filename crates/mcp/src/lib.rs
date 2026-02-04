//! ZED42 MCP Bridge - Model Context Protocol Integration
//!
//! Copyright (c) 2025 ZED42 Team. All rights reserved.
//! This software is proprietary. See LICENSE file for terms.
//!
//! Tool layer providing IDE connectors, file system operations,
//! git integration, build systems, and execution sandboxes.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod connectors;
pub mod filesystem;
pub mod git;
pub mod build;
pub mod sandbox;

/// MCP tool execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    pub agent_id: uuid::Uuid,
    pub session_id: uuid::Uuid,
    pub workspace_path: std::path::PathBuf,
}

/// MCP bridge for tool execution
pub struct McpBridge {
    context: ToolContext,
}

impl McpBridge {
    pub fn new(context: ToolContext) -> Self {
        Self { context }
    }

    pub fn context(&self) -> &ToolContext {
        &self.context
    }
}
