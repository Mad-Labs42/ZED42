//! File manipulation tools with sandboxed path security
//!
//! These tools provide safe file operations that are restricted
//! to the project sandbox directory to prevent path traversal attacks.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use crate::{Tool, ToolResult};

/// Path sanitizer for securing file operations
#[derive(Debug, Clone)]
pub struct PathSanitizer {
    /// The base project directory (sandbox root)
    sandbox_root: PathBuf,
}

impl PathSanitizer {
    /// Create a new path sanitizer
    ///
    /// # Arguments
    /// - `sandbox_root` - The root directory that all file operations are restricted to
    pub fn new(sandbox_root: impl Into<PathBuf>) -> Self {
        Self {
            sandbox_root: sandbox_root.into(),
        }
    }

    /// Sanitize a path to ensure it's within the sandbox
    ///
    /// Returns an error if:
    /// - Path contains ".." traversal
    /// - Path resolves outside the sandbox root
    pub fn sanitize(&self, path: &str) -> std::result::Result<PathBuf, String> {
        // Check for obvious traversal attempts
        if path.contains("..") {
            return Err("Path traversal (..) is not allowed".to_string());
        }

        // Build the full path
        let full_path = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.sandbox_root.join(path)
        };

        // Canonicalize to resolve symlinks and normalize
        let canonical = match full_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // For new files that don't exist yet, check parent
                if let Some(parent) = full_path.parent() {
                    if parent.exists() {
                        // Parent exists, check if it's in sandbox
                        let canonical_parent = parent.canonicalize()
                            .map_err(|e| format!("Cannot resolve parent path: {}", e))?;
                        if !canonical_parent.starts_with(&self.sandbox_root) {
                            return Err(format!(
                                "Path '{}' is outside the sandbox",
                                path
                            ));
                        }
                        return Ok(full_path);
                    }
                }
                return Err(format!("Path '{}' does not exist and cannot be verified", path));
            }
        };

        // Verify the canonical path is within sandbox
        if !canonical.starts_with(&self.sandbox_root) {
            return Err(format!(
                "Path '{}' resolves outside the sandbox",
                path
            ));
        }

        Ok(canonical)
    }
}

/// Parameters for ReadFile tool
#[derive(Debug, Deserialize)]
pub struct ReadFileParams {
    /// Path to the file to read (relative to sandbox root)
    pub path: String,
}

/// ReadFile tool - reads file contents safely
pub struct ReadFile {
    sanitizer: PathSanitizer,
}

impl ReadFile {
    pub fn new(sandbox_root: impl Into<PathBuf>) -> Self {
        Self {
            sanitizer: PathSanitizer::new(sandbox_root),
        }
    }
}

#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file within the project sandbox"
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file (relative to project root)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult {
        let params: ReadFileParams = serde_json::from_value(params)
            .map_err(|e| anyhow::anyhow!("Invalid parameters: {}", e))?;

        let safe_path = self.sanitizer.sanitize(&params.path)
            .map_err(|e| anyhow::anyhow!("Path security error: {}", e))?;

        let content = tokio::fs::read_to_string(&safe_path).await
            .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

        Ok(json!({
            "success": true,
            "path": safe_path.to_string_lossy(),
            "content": content,
            "size_bytes": content.len()
        }))
    }
}

/// Parameters for WriteFile tool
#[derive(Debug, Deserialize)]
pub struct WriteFileParams {
    /// Path to the file to write (relative to sandbox root)
    pub path: String,
    /// Content to write
    pub content: String,
    /// Whether to create parent directories
    #[serde(default)]
    pub create_dirs: bool,
}

/// WriteFile tool - writes file contents safely
pub struct WriteFile {
    sanitizer: PathSanitizer,
}

impl WriteFile {
    pub fn new(sandbox_root: impl Into<PathBuf>) -> Self {
        Self {
            sanitizer: PathSanitizer::new(sandbox_root),
        }
    }
}

#[async_trait]
impl Tool for WriteFile {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file within the project sandbox"
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file (relative to project root)"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "Whether to create parent directories if they don't exist",
                    "default": false
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult {
        let params: WriteFileParams = serde_json::from_value(params)
            .map_err(|e| anyhow::anyhow!("Invalid parameters: {}", e))?;

        // For write, we need to handle non-existent files
        // First check for path traversal directly
        if params.path.contains("..") {
            return Err(anyhow::anyhow!("Path traversal (..) is not allowed"));
        }

        // Use FileStateGuard for atomic write (Shadow Write pattern)
        // 1. Guard initializes .tmp file
        let guard = crate::fs_guard::FileStateGuard::new(&self.sanitizer, &params.path)
            .map_err(|e| anyhow::anyhow!("Failed to initialize atomic write guard: {}", e))?;

        let full_path = self.sanitizer.sandbox_root.join(&params.path);

        // Create parent directories if requested
        if params.create_dirs {
            if let Some(parent) = full_path.parent() {
                // Verify parent is still in sandbox
                let sandbox_canon = self.sanitizer.sandbox_root.canonicalize()
                    .map_err(|e| anyhow::anyhow!("Sandbox path error: {}", e))?;
                
                // Create the dirs first, then verify
                tokio::fs::create_dir_all(parent).await
                    .map_err(|e| anyhow::anyhow!("Failed to create directories: {}", e))?;
                
                let parent_canon = parent.canonicalize()
                    .map_err(|e| anyhow::anyhow!("Path resolution error: {}", e))?;
                
                if !parent_canon.starts_with(&sandbox_canon) {
                    return Err(anyhow::anyhow!("Path resolves outside sandbox"));
                }
            }
        }

        // 2. Write to the temporary path provided by guard
        tokio::fs::write(guard.path(), &params.content).await
            .map_err(|e| anyhow::anyhow!("Failed to write to temp file: {}", e))?;

        // 3. Commit the guard (Atomic update)
        guard.commit()
            .map_err(|e| anyhow::anyhow!("Failed to commit atomic write: {}", e))?;

        Ok(json!({
            "success": true,
            "path": full_path.to_string_lossy(),
            "bytes_written": params.content.len()
        }))
    }
}

/// Parameters for MoveFile tool
#[derive(Debug, Deserialize)]
pub struct MoveFileParams {
    pub source_path: String,
    pub target_path: String,
}

pub struct MoveFile {
    sanitizer: PathSanitizer,
}

impl MoveFile {
    pub fn new(sandbox_root: impl Into<PathBuf>) -> Self {
        Self { sanitizer: PathSanitizer::new(sandbox_root) }
    }
}

#[async_trait]
impl Tool for MoveFile {
    fn name(&self) -> &str { "move_file" }
    fn description(&self) -> &str { "Move or rename a file safely" }
    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "source_path": { "type": "string" },
                "target_path": { "type": "string" }
            },
            "required": ["source_path", "target_path"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult {
        let params: MoveFileParams = serde_json::from_value(params)?;
        let src = self.sanitizer.sanitize(&params.source_path)?;
        let dst = self.sanitizer.sanitize(&params.target_path)?;

        // check health before write (rename is a write to dir)
        let guard = crate::fs_guard::FileStateGuard::new(&self.sanitizer, &params.target_path)?;

        tokio::fs::rename(src, dst).await?;
        guard.commit()?;
        Ok(json!({ "success": true }))
    }
}

/// Parameters for DeleteFile tool
#[derive(Debug, Deserialize)]
pub struct DeleteFileParams {
    pub path: String,
}

pub struct DeleteFile {
    sanitizer: PathSanitizer,
}

impl DeleteFile {
    pub fn new(sandbox_root: impl Into<PathBuf>) -> Self {
        Self { sanitizer: PathSanitizer::new(sandbox_root) }
    }
}

#[async_trait]
impl Tool for DeleteFile {
    fn name(&self) -> &str { "delete_file" }
    fn description(&self) -> &str { "Delete a file safely" }
    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "path": { "type": "string" } },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult {
        let params: DeleteFileParams = serde_json::from_value(params)?;
        let path = self.sanitizer.sanitize(&params.path)?;
        
        // health check and guard
        let guard = crate::fs_guard::FileStateGuard::new(&self.sanitizer, &params.path)?;

        tokio::fs::remove_file(path).await?;
        guard.commit()?;
        Ok(json!({ "success": true }))
    }
}

/// Parameters for CreateDir tool
#[derive(Debug, Deserialize)]
pub struct CreateDirParams {
    pub path: String,
}

pub struct CreateDir {
    sanitizer: PathSanitizer,
}

impl CreateDir {
    pub fn new(sandbox_root: impl Into<PathBuf>) -> Self {
        Self { sanitizer: PathSanitizer::new(sandbox_root) }
    }
}

#[async_trait]
impl Tool for CreateDir {
    fn name(&self) -> &str { "create_dir" }
    fn description(&self) -> &str { "Create a directory safely" }
    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "path": { "type": "string" } },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult {
        let params: CreateDirParams = serde_json::from_value(params)?;
        
        // Use FileStateGuard for atomic dir creation
        let guard = crate::fs_guard::FileStateGuard::new_dir(&self.sanitizer, &params.path)?;
        
        tokio::fs::create_dir_all(guard.path()).await?;
        guard.commit()?;
        
        Ok(json!({ "success": true }))
    }
}

/// Parameters for DeleteDir tool
#[derive(Debug, Deserialize)]
pub struct DeleteDirParams {
    pub path: String,
    pub recursive: bool,
}

pub struct DeleteDir {
    sanitizer: PathSanitizer,
}

impl DeleteDir {
    pub fn new(sandbox_root: impl Into<PathBuf>) -> Self {
        Self { sanitizer: PathSanitizer::new(sandbox_root) }
    }
}

#[async_trait]
impl Tool for DeleteDir {
    fn name(&self) -> &str { "delete_dir" }
    fn description(&self) -> &str { "Delete a directory safely" }
    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { 
                "path": { "type": "string" },
                "recursive": { "type": "boolean", "default": false }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult {
        let params: DeleteDirParams = serde_json::from_value(params)?;
        let path = self.sanitizer.sanitize(&params.path)?;
        
        // health check and guard
        let guard = crate::fs_guard::FileStateGuard::new_dir(&self.sanitizer, &params.path)?;

        if params.recursive {
            tokio::fs::remove_dir_all(path).await?;
        } else {
            tokio::fs::remove_dir(path).await?;
        }
        guard.commit()?;
        Ok(json!({ "success": true }))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_path_sanitizer_rejects_traversal() {
        let temp = tempdir().unwrap();
        let sanitizer = PathSanitizer::new(temp.path());

        // Should reject path traversal
        assert!(sanitizer.sanitize("../etc/passwd").is_err());
        assert!(sanitizer.sanitize("foo/../../../etc/passwd").is_err());
        assert!(sanitizer.sanitize("..").is_err());
    }

    #[test]
    fn test_path_sanitizer_accepts_valid_paths() {
        let temp = tempdir().unwrap();
        let sanitizer = PathSanitizer::new(temp.path());

        // Create a file to test
        std::fs::write(temp.path().join("test.txt"), "hello").unwrap();

        // Should accept valid paths
        let result = sanitizer.sanitize("test.txt");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_read_file_tool() {
        let temp = tempdir().unwrap();
        std::fs::write(temp.path().join("test.txt"), "hello world").unwrap();

        let tool = ReadFile::new(temp.path());
        let result = tool.execute(json!({ "path": "test.txt" })).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["success"], true);
        assert_eq!(value["content"], "hello world");
    }

    #[tokio::test]
    async fn test_write_file_tool() {
        let temp = tempdir().unwrap();
        let tool = WriteFile::new(temp.path());

        let result = tool.execute(json!({
            "path": "output.txt",
            "content": "generated code"
        })).await;

        assert!(result.is_ok());

        // Verify file was written
        let content = std::fs::read_to_string(temp.path().join("output.txt")).unwrap();
        assert_eq!(content, "generated code");
    }

    #[tokio::test]
    async fn test_write_file_rejects_traversal() {
        let temp = tempdir().unwrap();
        let tool = WriteFile::new(temp.path());

        let result = tool.execute(json!({
            "path": "../evil.txt",
            "content": "bad stuff"
        })).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_dir_tool() {
        let temp = tempdir().unwrap();
        std::fs::create_dir(temp.path().join("subdir")).unwrap();
        std::fs::write(temp.path().join("file1.txt"), "content").unwrap();
        std::fs::write(temp.path().join("subdir/file2.txt"), "content").unwrap();

        let tool = ListDir::new(temp.path());
        let result = tool.execute(json!({ "path": "." })).await;

        assert!(result.is_ok());
        let val = result.unwrap();
        let entries = val["entries"].as_array().unwrap();
        
        // Should find file1.txt and subdir
        assert!(entries.iter().any(|e| e["name"] == "file1.txt" && e["type"] == "file"));
        assert!(entries.iter().any(|e| e["name"] == "subdir" && e["type"] == "dir"));
    }
}

/// Parameters for ListDir tool
#[derive(Debug, Deserialize)]
pub struct ListDirParams {
    /// Directory path to list (relative to sandbox root)
    pub path: String,
}

/// ListDir tool - lists directory contents safely
pub struct ListDir {
    sanitizer: PathSanitizer,
}

impl ListDir {
    pub fn new(sandbox_root: impl Into<PathBuf>) -> Self {
        Self {
            sanitizer: PathSanitizer::new(sandbox_root),
        }
    }
}

#[async_trait]
impl Tool for ListDir {
    fn name(&self) -> &str {
        "list_dir"
    }

    fn description(&self) -> &str {
        "List contents of a directory within the project sandbox"
    }

    fn parameter_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory (relative to project root)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult {
        let params: ListDirParams = serde_json::from_value(params)
            .map_err(|e| anyhow::anyhow!("Invalid parameters: {}", e))?;

        let safe_path = self.sanitizer.sanitize(&params.path)
            .map_err(|e| anyhow::anyhow!("Path security error: {}", e))?;

        if !safe_path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory: {}", params.path));
        }

        let mut read_dir = tokio::fs::read_dir(&safe_path).await
            .map_err(|e| anyhow::anyhow!("Failed to read directory: {}", e))?;

        let mut entries = Vec::new();
        while let Some(entry) = read_dir.next_entry().await
            .map_err(|e| anyhow::anyhow!("Failed to iterate entries: {}", e))? 
        {
            let metadata = entry.metadata().await
                .map_err(|e| anyhow::anyhow!("Failed to get metadata: {}", e))?;
            
            let file_type = if metadata.is_dir() { "dir" } 
                else if metadata.is_symlink() { "symlink" } 
                else { "file" };

            entries.push(json!({
                "name": entry.file_name().to_string_lossy(),
                "type": file_type,
                "size": metadata.len(),
                "readonly": metadata.permissions().readonly(),
            }));
        }

        //Sort entries by name for deterministic output
        entries.sort_by(|a, b| a["name"].as_str().unwrap().cmp(b["name"].as_str().unwrap()));

        Ok(json!({
            "success": true,
            "path": safe_path.to_string_lossy(),
            "entries": entries
        }))
    }
}
