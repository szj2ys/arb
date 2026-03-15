//! Tool system for AI Agent
//!
//! Provides tools for file operations, shell execution, and git commands.
//! Tools can be called by the AI agent to perform actions in the environment.

pub use crate::ai::provider::ToolDefinition;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error.into()),
        }
    }
}

/// Tool trait - implemented by all tools
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool definition (for LLM function calling)
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool with the given arguments
    async fn execute(&self, args: Value) -> ToolResult;

    /// Get the tool name
    fn name(&self) -> &str;
}

/// Create a tool definition with the given name and description
pub fn tool_def(name: impl Into<String>, description: impl Into<String>) -> ToolDefinition {
    ToolDefinition {
        name: name.into(),
        description: description.into(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

/// Add a parameter to the tool definition
pub fn with_parameter(
    mut def: ToolDefinition,
    name: impl Into<String>,
    param_type: impl Into<String>,
    description: impl Into<String>,
    required: bool,
) -> ToolDefinition {
    let name = name.into();
    let mut properties = def
        .parameters
        .get_mut("properties")
        .and_then(|p| p.as_object_mut())
        .expect("parameters should be an object")
        .clone();

    properties.insert(
        name.clone(),
        serde_json::json!({
            "type": param_type.into(),
            "description": description.into(),
        }),
    );

    if let Some(obj) = def.parameters.as_object_mut() {
        obj.insert(
            "properties".to_string(),
            serde_json::Value::Object(properties),
        );

        if required {
            let required = obj
                .get_mut("required")
                .and_then(|r| r.as_array_mut())
                .expect("required should be an array");
            required.push(serde_json::json!(name));
        }
    }

    def
}

/// File read tool
pub struct FileReadTool {
    working_dir: PathBuf,
}

impl FileReadTool {
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn definition(&self) -> ToolDefinition {
        let def = tool_def("file_read", "Read the contents of a file at the given path");
        let def = with_parameter(def, "path", "string", "The path to the file to read", true);
        let def = with_parameter(
            def,
            "limit",
            "integer",
            "Maximum number of lines to read (optional)",
            false,
        );
        with_parameter(
            def,
            "offset",
            "integer",
            "Line number to start reading from (optional)",
            false,
        )
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let path = match args.get("path").and_then(|p| p.as_str()) {
            Some(p) => p,
            None => return ToolResult::error("Missing required parameter 'path'"),
        };

        let file_path = self.working_dir.join(path);

        // Security check: ensure the path is within the working directory
        if let Ok(canonical_path) = file_path.canonicalize() {
            if let Ok(canonical_working) = self.working_dir.canonicalize() {
                if !canonical_path.starts_with(&canonical_working) {
                    return ToolResult::error("Path is outside the working directory");
                }
            }
        }

        // Read file
        match tokio::fs::read_to_string(&file_path).await {
            Ok(content) => {
                // Apply offset and limit if specified
                let offset = args.get("offset").and_then(|o| o.as_u64()).unwrap_or(0) as usize;
                let limit = args
                    .get("limit")
                    .and_then(|l| l.as_u64())
                    .unwrap_or(u64::MAX) as usize;

                if offset > 0 || limit < usize::MAX {
                    let lines: Vec<&str> = content.lines().collect();
                    let start = offset.saturating_sub(1);
                    let end = (start + limit).min(lines.len());
                    let selected: Vec<&str> = lines[start..end].to_vec();
                    ToolResult::success(selected.join("\n"))
                } else {
                    ToolResult::success(content)
                }
            }
            Err(e) => ToolResult::error(format!("Failed to read file: {}", e)),
        }
    }
}

/// File write tool
pub struct FileWriteTool {
    working_dir: PathBuf,
}

impl FileWriteTool {
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn definition(&self) -> ToolDefinition {
        let def = tool_def(
            "file_write",
            "Write content to a file at the given path. Creates the file if it doesn't exist.",
        );
        let def = with_parameter(def, "path", "string", "The path to the file to write", true);
        with_parameter(
            def,
            "content",
            "string",
            "The content to write to the file",
            true,
        )
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let path = match args.get("path").and_then(|p| p.as_str()) {
            Some(p) => p,
            None => return ToolResult::error("Missing required parameter 'path'"),
        };

        let content = match args.get("content").and_then(|c| c.as_str()) {
            Some(c) => c,
            None => return ToolResult::error("Missing required parameter 'content'"),
        };

        let file_path = self.working_dir.join(path);

        // Security check: ensure the path is within the working directory
        if let Some(parent) = file_path.parent() {
            if let Ok(canonical_parent) = parent.canonicalize() {
                if let Ok(canonical_working) = self.working_dir.canonicalize() {
                    if !canonical_parent.starts_with(&canonical_working) {
                        return ToolResult::error("Path is outside the working directory");
                    }
                }
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return ToolResult::error(format!("Failed to create directory: {}", e));
            }
        }

        // Write file
        match tokio::fs::write(&file_path, content).await {
            Ok(_) => ToolResult::success(format!("File written successfully: {}", path)),
            Err(e) => ToolResult::error(format!("Failed to write file: {}", e)),
        }
    }
}

/// File search tool (grep-like)
pub struct FileSearchTool {
    working_dir: PathBuf,
}

impl FileSearchTool {
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for FileSearchTool {
    fn name(&self) -> &str {
        "file_search"
    }

    fn definition(&self) -> ToolDefinition {
        let def = tool_def(
            "file_search",
            "Search for files matching a pattern, or search file contents for text",
        );
        let def = with_parameter(
            def,
            "pattern",
            "string",
            "The search pattern (regex for content, glob for files)",
            true,
        );
        let def = with_parameter(
            def,
            "path",
            "string",
            "The directory or file to search in (default: working directory)",
            false,
        );
        let def = with_parameter(
            def,
            "search_type",
            "string",
            "Type of search: 'content' (grep) or 'files' (find/glob). Default: 'content'",
            false,
        );
        with_parameter(
            def,
            "glob",
            "string",
            "File glob pattern to filter files (e.g., '*.rs'). Only for content search.",
            false,
        )
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let pattern = match args.get("pattern").and_then(|p| p.as_str()) {
            Some(p) => p,
            None => return ToolResult::error("Missing required parameter 'pattern'"),
        };

        let search_type = args
            .get("search_type")
            .and_then(|s| s.as_str())
            .unwrap_or("content");

        let search_path = args
            .get("path")
            .and_then(|p| p.as_str())
            .map(|p| self.working_dir.join(p))
            .unwrap_or_else(|| self.working_dir.clone());

        match search_type {
            "files" => {
                // Use glob pattern matching
                let pattern = search_path.join(pattern);
                let pattern_str = pattern.to_string_lossy();

                match glob::glob(&pattern_str) {
                    Ok(entries) => {
                        let files: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .map(|p| {
                                p.strip_prefix(&self.working_dir)
                                    .unwrap_or(&p)
                                    .display()
                                    .to_string()
                            })
                            .collect();

                        if files.is_empty() {
                            ToolResult::success("No files found matching the pattern")
                        } else {
                            ToolResult::success(files.join("\n"))
                        }
                    }
                    Err(e) => ToolResult::error(format!("Invalid glob pattern: {}", e)),
                }
            }
            "content" => {
                // Use grep-like search
                let glob_pattern = args.get("glob").and_then(|g| g.as_str()).unwrap_or("*");

                // Use ripgrep if available, otherwise fall back to grep
                let output = Command::new("rg")
                    .args([
                        "--line-number",
                        "--with-filename",
                        "--fixed-strings",
                        "--color=never",
                        "-g",
                        glob_pattern,
                        pattern,
                    ])
                    .current_dir(&self.working_dir)
                    .output()
                    .await;

                match output {
                    Ok(result) => {
                        let stdout = String::from_utf8_lossy(&result.stdout);
                        let stderr = String::from_utf8_lossy(&result.stderr);

                        if result.status.success() || !stdout.is_empty() {
                            if stdout.is_empty() {
                                ToolResult::success("No matches found")
                            } else {
                                ToolResult::success(stdout.to_string())
                            }
                        } else {
                            // Try with grep as fallback
                            let grep_output = Command::new("grep")
                                .args(["-r", "-n", "--include", glob_pattern, pattern, "."])
                                .current_dir(&self.working_dir)
                                .output()
                                .await;

                            match grep_output {
                                Ok(grep_result) => {
                                    let grep_stdout = String::from_utf8_lossy(&grep_result.stdout);
                                    if grep_stdout.is_empty() {
                                        ToolResult::success("No matches found")
                                    } else {
                                        ToolResult::success(grep_stdout.to_string())
                                    }
                                }
                                Err(_) => ToolResult::error(format!("Search failed: {}", stderr)),
                            }
                        }
                    }
                    Err(_) => {
                        // Fall back to grep
                        let grep_output = Command::new("grep")
                            .args(["-r", "-n", "--include", glob_pattern, pattern, "."])
                            .current_dir(&self.working_dir)
                            .output()
                            .await;

                        match grep_output {
                            Ok(grep_result) => {
                                let grep_stdout = String::from_utf8_lossy(&grep_result.stdout);
                                if grep_stdout.is_empty() {
                                    ToolResult::success("No matches found")
                                } else {
                                    ToolResult::success(grep_stdout.to_string())
                                }
                            }
                            Err(e) => ToolResult::error(format!("Search failed: {}", e)),
                        }
                    }
                }
            }
            _ => ToolResult::error("Invalid search_type. Use 'content' or 'files'"),
        }
    }
}

/// Shell command execution tool
pub struct ShellTool {
    working_dir: PathBuf,
    allowed_commands: Vec<String>,
}

impl ShellTool {
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
            allowed_commands: vec![],
        }
    }

    /// Set allowed commands (empty = allow all)
    #[allow(dead_code)]
    pub fn with_allowed_commands(mut self, commands: Vec<String>) -> Self {
        self.allowed_commands = commands;
        self
    }

    fn is_command_allowed(&self, command: &str) -> bool {
        if self.allowed_commands.is_empty() {
            return true;
        }

        let cmd = command.split_whitespace().next().unwrap_or("");
        self.allowed_commands.iter().any(|allowed| allowed == cmd)
    }
}

#[async_trait::async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn definition(&self) -> ToolDefinition {
        let def = tool_def("shell", "Execute a shell command and return the output");
        let def = with_parameter(
            def,
            "command",
            "string",
            "The shell command to execute",
            true,
        );
        let def = with_parameter(
            def,
            "timeout",
            "integer",
            "Timeout in seconds (default: 60)",
            false,
        );
        with_parameter(
            def,
            "working_dir",
            "string",
            "Working directory for the command (default: current)",
            false,
        )
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let command = match args.get("command").and_then(|c| c.as_str()) {
            Some(c) => c,
            None => return ToolResult::error("Missing required parameter 'command'"),
        };

        // Security check: validate command
        if !self.is_command_allowed(command) {
            return ToolResult::error(format!(
                "Command not allowed. Allowed commands: {:?}",
                self.allowed_commands
            ));
        }

        let timeout_secs = args.get("timeout").and_then(|t| t.as_u64()).unwrap_or(60);

        let working_dir = args
            .get("working_dir")
            .and_then(|w| w.as_str())
            .map(|w| self.working_dir.join(w))
            .unwrap_or_else(|| self.working_dir.clone());

        // Execute command with timeout
        let result = tokio::time::timeout(
            tokio::time::Duration::from_secs(timeout_secs),
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(&working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let mut result_text = String::new();
                if !stdout.is_empty() {
                    result_text.push_str(&format!("STDOUT:\n{}\n", stdout));
                }
                if !stderr.is_empty() {
                    result_text.push_str(&format!("STDERR:\n{}\n", stderr));
                }

                if output.status.success() {
                    ToolResult::success(result_text)
                } else {
                    let exit_code = output
                        .status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "signal".to_string());
                    result_text.push_str(&format!("Exit code: {}\n", exit_code));
                    ToolResult {
                        success: false,
                        output: result_text,
                        error: Some(format!("Command failed with exit code {}", exit_code)),
                    }
                }
            }
            Ok(Err(e)) => ToolResult::error(format!("Failed to execute command: {}", e)),
            Err(_) => {
                ToolResult::error(format!("Command timed out after {} seconds", timeout_secs))
            }
        }
    }
}

/// Git tool for git operations
pub struct GitTool {
    working_dir: PathBuf,
}

impl GitTool {
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for GitTool {
    fn name(&self) -> &str {
        "git"
    }

    fn definition(&self) -> ToolDefinition {
        let def = tool_def("git", "Execute git commands");
        let def = with_parameter(
            def,
            "subcommand",
            "string",
            "The git subcommand to run (status, log, diff, branch, etc.)",
            true,
        );
        with_parameter(
            def,
            "args",
            "array",
            "Additional arguments for the git command",
            false,
        )
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let subcommand = match args.get("subcommand").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => return ToolResult::error("Missing required parameter 'subcommand'"),
        };

        // Build command
        let mut cmd = Command::new("git");
        cmd.arg(subcommand).current_dir(&self.working_dir);

        // Add additional arguments
        if let Some(additional_args) = args.get("args").and_then(|a| a.as_array()) {
            for arg in additional_args {
                if let Some(arg_str) = arg.as_str() {
                    cmd.arg(arg_str);
                }
            }
        }

        // Execute
        match cmd.output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let mut result_text = String::new();
                if !stdout.is_empty() {
                    result_text.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    result_text.push_str(&stderr);
                }

                if output.status.success() {
                    ToolResult::success(result_text)
                } else {
                    ToolResult {
                        success: false,
                        output: result_text,
                        error: Some(format!("Git command failed: {}", stderr)),
                    }
                }
            }
            Err(e) => ToolResult::error(format!("Failed to execute git command: {}", e)),
        }
    }
}

/// List directory tool
pub struct ListDirectoryTool {
    working_dir: PathBuf,
}

impl ListDirectoryTool {
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn definition(&self) -> ToolDefinition {
        let def = tool_def("list_directory", "List the contents of a directory");
        let def = with_parameter(
            def,
            "path",
            "string",
            "The directory path to list (default: current)",
            false,
        );
        with_parameter(
            def,
            "recursive",
            "boolean",
            "Whether to list recursively",
            false,
        )
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let path = args
            .get("path")
            .and_then(|p| p.as_str())
            .map(|p| self.working_dir.join(p))
            .unwrap_or_else(|| self.working_dir.clone());

        let recursive = args
            .get("recursive")
            .and_then(|r| r.as_bool())
            .unwrap_or(false);

        // Security check
        if let Ok(canonical_path) = path.canonicalize() {
            if let Ok(canonical_working) = self.working_dir.canonicalize() {
                if !canonical_path.starts_with(&canonical_working) {
                    return ToolResult::error("Path is outside the working directory");
                }
            }
        }

        let mut entries = Vec::new();

        if recursive {
            match walk_directory(&path, &self.working_dir).await {
                Ok(e) => entries = e,
                Err(e) => return ToolResult::error(format!("Failed to read directory: {}", e)),
            }
        } else {
            match tokio::fs::read_dir(&path).await {
                Ok(mut dir) => {
                    while let Ok(Some(entry)) = dir.next_entry().await {
                        let file_type = entry.file_type().await.ok();
                        let prefix = if file_type.map(|ft| ft.is_dir()).unwrap_or(false) {
                            "[DIR] "
                        } else if file_type.map(|ft| ft.is_symlink()).unwrap_or(false) {
                            "[LINK] "
                        } else {
                            "[FILE] "
                        };

                        let _name = entry.file_name().to_string_lossy().to_string();
                        let relative_path = entry
                            .path()
                            .strip_prefix(&self.working_dir)
                            .unwrap_or(&entry.path())
                            .display()
                            .to_string();

                        entries.push(format!("{}{}", prefix, relative_path));
                    }
                }
                Err(e) => return ToolResult::error(format!("Failed to read directory: {}", e)),
            }
        }

        entries.sort();
        ToolResult::success(entries.join("\n"))
    }
}

async fn walk_directory(path: &Path, base_dir: &Path) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    let mut stack = vec![path.to_path_buf()];

    while let Some(current) = stack.pop() {
        match tokio::fs::read_dir(&current).await {
            Ok(mut dir) => {
                while let Ok(Some(entry)) = dir.next_entry().await {
                    let file_type = entry.file_type().await.ok();
                    let prefix = if file_type.map(|ft| ft.is_dir()).unwrap_or(false) {
                        "[DIR] "
                    } else if file_type.map(|ft| ft.is_symlink()).unwrap_or(false) {
                        "[LINK] "
                    } else {
                        "[FILE] "
                    };

                    let relative_path = entry
                        .path()
                        .strip_prefix(base_dir)
                        .unwrap_or(&entry.path())
                        .display()
                        .to_string();

                    entries.push(format!("{}{}", prefix, relative_path));

                    // Add directories to stack for recursion
                    if file_type.map(|ft| ft.is_dir()).unwrap_or(false) {
                        stack.push(entry.path());
                    }
                }
            }
            Err(_) => continue,
        }
    }

    Ok(entries)
}

/// Tool registry - manages all available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new tool registry with default tools
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        let working_dir = working_dir.into();
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register default tools
        registry.register(Box::new(FileReadTool::new(&working_dir)));
        registry.register(Box::new(FileWriteTool::new(&working_dir)));
        registry.register(Box::new(FileSearchTool::new(&working_dir)));
        registry.register(Box::new(ShellTool::new(&working_dir)));
        registry.register(Box::new(GitTool::new(&working_dir)));
        registry.register(Box::new(ListDirectoryTool::new(&working_dir)));

        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Get all tool definitions
    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Execute a tool by name with arguments
    pub async fn execute(&self, name: &str, args: Value) -> ToolResult {
        match self.get(name) {
            Some(tool) => tool.execute(args).await,
            None => ToolResult::error(format!("Tool '{}' not found", name)),
        }
    }

    /// Get all tool names
    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|k| k.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition_builder() {
        let def = tool_def("test_tool", "A test tool");
        let def = with_parameter(def, "param1", "string", "First parameter", true);
        let def = with_parameter(def, "param2", "integer", "Second parameter", false);

        assert_eq!(def.name, "test_tool");
        assert_eq!(def.description, "A test tool");
        assert!(def
            .parameters
            .get("properties")
            .unwrap()
            .get("param1")
            .is_some());
        assert!(def
            .parameters
            .get("properties")
            .unwrap()
            .get("param2")
            .is_some());
    }

    #[test]
    fn test_tool_result() {
        let success = ToolResult::success("output");
        assert!(success.success);
        assert_eq!(success.output, "output");
        assert!(success.error.is_none());

        let error = ToolResult::error("error message");
        assert!(!error.success);
        assert_eq!(error.error, Some("error message".to_string()));
    }
}
