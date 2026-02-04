//! ZED42 Toolboxes - Structured Capability Distribution
//!
//! Copyright (c) 2025 ZED42 Team. All rights reserved.
//! This software is proprietary. See LICENSE file for terms.
//!
//! Tools are capabilities, not free-for-all functions. Each agent receives
//! a curated toolbox aligned with their role and team mandate.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod code;
pub mod analysis;
pub mod testing;
pub mod vcs;
pub mod external;
pub mod visualization;
pub mod file_manipulation;
pub mod shell;
pub mod fs_guard;

/// Tool execution result
pub type ToolResult = anyhow::Result<serde_json::Value>;

/// Base trait for all tools
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name identifier
    fn name(&self) -> &str;

    /// Tool description for LLM context
    fn description(&self) -> &str;

    /// JSON schema for tool parameters
    fn parameter_schema(&self) -> serde_json::Value;

    /// Execute the tool with given parameters
    async fn execute(&self, params: serde_json::Value) -> ToolResult;
}

/// Toolbox containing a set of tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toolbox {
    pub name: String,
    pub tools: Vec<String>,
}

/// Toolbox registry
pub struct ToolboxRegistry {
    toolboxes: HashMap<String, Toolbox>,
}

impl ToolboxRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            toolboxes: HashMap::new(),
        };
        registry.initialize_standard_toolboxes();
        registry
    }

    fn initialize_standard_toolboxes(&mut self) {
        // Code Manipulation Toolboxes
        self.register(Toolbox {
            name: "CodeGeneration".to_string(),
            tools: vec![
                "generate_function".to_string(),
                "generate_class".to_string(),
                "generate_module".to_string(),
                "complete_code".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "FileManipulation".to_string(),
            tools: vec![
                "read_file".to_string(),
                "write_file".to_string(),
                "move_file".to_string(),
                "delete_file".to_string(),
                "list_dir".to_string(),
            ],
        });

        // System/Shell Toolboxes
        self.register(Toolbox {
            name: "Shell".to_string(),
            tools: vec![
                "execute_command".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "Refactoring".to_string(),
            tools: vec![
                "rename_symbol".to_string(),
                "extract_function".to_string(),
                "inline_function".to_string(),
                "move_symbol".to_string(),
            ],
        });

        // Analysis Toolboxes
        self.register(Toolbox {
            name: "StaticAnalysis".to_string(),
            tools: vec![
                "parse_ast".to_string(),
                "detect_vulnerabilities".to_string(),
                "find_code_smells".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "DependencyScanning".to_string(),
            tools: vec![
                "list_dependencies".to_string(),
                "check_vulnerabilities".to_string(),
                "check_licenses".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "MetricsAnalysis".to_string(),
            tools: vec![
                "calculate_complexity".to_string(),
                "measure_maintainability".to_string(),
                "score_technical_debt".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "GraphQuery".to_string(),
            tools: vec![
                "traverse_dependencies".to_string(),
                "analyze_impact".to_string(),
                "find_patterns".to_string(),
            ],
        });

        // Testing & Validation Toolboxes
        self.register(Toolbox {
            name: "Testing".to_string(),
            tools: vec![
                "generate_unit_tests".to_string(),
                "run_tests".to_string(),
                "measure_coverage".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "Fuzzing".to_string(),
            tools: vec![
                "generate_inputs".to_string(),
                "detect_crashes".to_string(),
                "find_edge_cases".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "PerformanceProfiling".to_string(),
            tools: vec![
                "profile_cpu".to_string(),
                "profile_memory".to_string(),
                "run_benchmarks".to_string(),
            ],
        });

        // Version Control Toolboxes
        self.register(Toolbox {
            name: "GitOperations".to_string(),
            tools: vec![
                "git_commit".to_string(),
                "git_branch".to_string(),
                "git_merge".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "GitHistory".to_string(),
            tools: vec![
                "git_blame".to_string(),
                "git_log".to_string(),
                "find_author".to_string(),
            ],
        });

        // External Integration Toolboxes
        self.register(Toolbox {
            name: "BuildSystem".to_string(),
            tools: vec![
                "compile".to_string(),
                "link".to_string(),
                "package".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "Sandboxing".to_string(),
            tools: vec![
                "create_sandbox".to_string(),
                "execute_in_sandbox".to_string(),
                "destroy_sandbox".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "ProcessManagement".to_string(),
            tools: vec![
                "spawn_process".to_string(),
                "monitor_process".to_string(),
                "kill_process".to_string(),
            ],
        });

        // Visualization Toolboxes
        self.register(Toolbox {
            name: "DiagramGeneration".to_string(),
            tools: vec![
                "generate_architecture_diagram".to_string(),
                "generate_sequence_diagram".to_string(),
                "generate_call_graph".to_string(),
            ],
        });

        self.register(Toolbox {
            name: "VisualizationGeneration".to_string(),
            tools: vec![
                "create_dashboard".to_string(),
                "generate_heatmap".to_string(),
                "visualize_dependencies".to_string(),
            ],
        });
    }

    pub fn register(&mut self, toolbox: Toolbox) {
        self.toolboxes.insert(toolbox.name.clone(), toolbox);
    }

    pub fn get(&self, name: &str) -> Option<&Toolbox> {
        self.toolboxes.get(name)
    }

    pub fn get_tools_for_agent(&self, toolbox_names: &[String]) -> Vec<String> {
        toolbox_names
            .iter()
            .filter_map(|name| self.get(name))
            .flat_map(|toolbox| toolbox.tools.clone())
            .collect()
    }
}

impl Default for ToolboxRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbox_registry() {
        let registry = ToolboxRegistry::new();
        let toolbox = registry.get("CodeGeneration");
        assert!(toolbox.is_some());
        assert!(!toolbox.unwrap().tools.is_empty());
    }

    #[test]
    fn test_agent_toolbox_retrieval() {
        let registry = ToolboxRegistry::new();
        let toolbox_names = vec![
            "CodeGeneration".to_string(),
            "Testing".to_string(),
        ];
        let tools = registry.get_tools_for_agent(&toolbox_names);
        assert!(!tools.is_empty());
    }
}
