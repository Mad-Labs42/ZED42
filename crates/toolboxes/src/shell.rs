//! Forensic Shell Execution Tool
//!
//! Provides command execution with strict output separation (stdout/stderr)
//! and structured feedback to enable high-fidelity agentic debugging.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::process::Command;
use crate::{Tool, ToolResult};

/// Parameters for ExecuteCommand tool
#[derive(Debug, Deserialize)]
pub struct ExecuteCommandParams {
    /// Command to execute (e.g., "git", "cargo")
    pub command: String,
    /// Arguments for the command
    pub args: Vec<String>,
    /// Working directory (optional, relative to sandbox root)
    pub cwd: Option<String>,
}

/// ExecuteCommand tool - runs shell commands with forensic logging
pub struct ExecuteCommand {
    /// Root directory for execution context
    sandbox_root: std::path::PathBuf,
}

impl ExecuteCommand {
    pub fn new(sandbox_root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            sandbox_root: sandbox_root.into(),
        }
    }
}

#[async_trait]
impl Tool for ExecuteCommand {
    fn name(&self) -> &str {
        "execute_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command with full output capture (stdout/stderr). Use this for git, cargo, or system tools."
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The executable to run (e.g., 'git')"
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Arguments to pass to the command"
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory relative to project root (optional)"
                }
            },
            "required": ["command", "args"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult {
        let params: ExecuteCommandParams = serde_json::from_value(params)
            .map_err(|e| anyhow::anyhow!("Invalid parameters: {}", e))?;

        // 1. Resolve Working Directory
        let cwd = if let Some(ref dir) = params.cwd {
            let full_path = self.sandbox_root.join(dir);
            // Verify path is safe (simple check here, deeper sanitization if FS tool logic was shared/exposed)
            // ideally we would reuse PathSanitizer but it's currently private to file_manipulation.
            // For now, we perform a canonicalization check.
            if full_path.to_string_lossy().contains("..") { // Basic pre-check
                 return Err(anyhow::anyhow!("Path traversal in cwd not allowed"));
            }
             match full_path.canonicalize() {
                Ok(p) => {
                     // Verify prefix
                    if !p.starts_with(&self.sandbox_root) {
                        return Err(anyhow::anyhow!("Working directory resolves outside sandbox"));
                    }
                    p
                },
                Err(e) => return Err(anyhow::anyhow!("Invalid working directory '{}': {}", dir, e)),
            }
        } else {
            self.sandbox_root.clone()
        };

        // 2. Prepare Command
        let mut child = Command::new(&params.command);
        child.args(&params.args)
             .current_dir(&cwd)
             .stdout(Stdio::piped())
             .stderr(Stdio::piped())
             .stdin(Stdio::null()); // No interactive input

        // 3. Execute (Hard Fail detection)
        let output = match child.output().await {
            Ok(o) => o,
            Err(e) => {
                // HARD FAIL: Binary not found, permission denied, etc.
                return Err(anyhow::anyhow!("Execution failed (Binary not found or IO error): {}", e));
            }
        };

        // 4. Process Output (Forensic Capture)
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        // 5. Structure Result (Soft Fail is still a Result::Ok with exit_code != 0)
        Ok(json!({
            "command": params.command,
            "args": params.args,
            "cwd": cwd.to_string_lossy(),
            "exit_code": exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "success": output.status.success()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_execute_echo_success() {
        let temp = tempdir().unwrap();
        let tool = ExecuteCommand::new(temp.path());
        
        // Windows 'echo' is intricate with Command::new, usually need 'cmd /c echo'
        // But for cross-platform robustness in tests, let's use 'cargo --version' which we know exists
        // assuming cargo is in path.
        let result = tool.execute(json!({
            "command": "cargo",
            "args": ["--version"],
        })).await;

        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["exit_code"], 0);
        assert!(val["stdout"].as_str().unwrap().contains("cargo"));
    }

    #[tokio::test]
    async fn test_hard_fail_binary_not_found() {
        let temp = tempdir().unwrap();
        let tool = ExecuteCommand::new(temp.path());

        let result = tool.execute(json!({
            "command": "non_existent_binary_xyz",
            "args": []
        })).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Execution failed"));
    }

    #[tokio::test]
    async fn test_cwd_sandboxing() {
        let temp = tempdir().unwrap();
        let tool = ExecuteCommand::new(temp.path());

        let result = tool.execute(json!({
            "command": "cargo",
            "args": ["--version"],
            "cwd": "../"
        })).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Path traversal"));
    }
}
