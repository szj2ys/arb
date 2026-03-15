//! Agent Mode for AI Terminal
//!
//! Provides autonomous task execution with tool use capabilities.
//! The agent can plan, execute, and iterate on tasks using available tools.

use crate::ai::provider::{ChatRequest, LLMProvider, Message, Role};
use crate::ai::tools::{ToolRegistry, ToolResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum number of iterations
    pub max_iterations: u32,
    /// Maximum tokens per request
    pub max_tokens: u32,
    /// Temperature for LLM
    pub temperature: f32,
    /// Working directory for the agent
    pub working_dir: PathBuf,
    /// Whether to ask for confirmation before executing shell commands
    pub confirm_shell_commands: bool,
    /// Whether to enable verbose logging
    pub verbose: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 30,
            max_tokens: 4096,
            temperature: 0.7,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            confirm_shell_commands: true,
            verbose: false,
        }
    }
}

/// Agent execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Agent execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub iteration: u32,
    pub thought: String,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<ToolResult>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Tool call from the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

/// Agent execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub success: bool,
    pub output: String,
    pub steps: Vec<AgentStep>,
    pub total_iterations: u32,
    pub duration_secs: u64,
}

/// Agent event for progress reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    Started {
        task: String,
    },
    Thinking {
        iteration: u32,
        thought: String,
    },
    ToolCalling {
        iteration: u32,
        tool_name: String,
        arguments: Value,
    },
    ToolResult {
        iteration: u32,
        result: ToolResult,
    },
    Completed {
        result: AgentResult,
    },
    Failed {
        error: String,
    },
    StatusChanged {
        status: AgentStatus,
    },
}

/// The Agent that can execute tasks autonomously
pub struct Agent {
    config: AgentConfig,
    provider: Arc<dyn LLMProvider>,
    tool_registry: Arc<ToolRegistry>,
    status: Arc<Mutex<AgentStatus>>,
    history: Arc<Mutex<Vec<AgentStep>>>,
    event_tx: Option<mpsc::UnboundedSender<AgentEvent>>,
}

impl Agent {
    /// Create a new agent with the given configuration
    pub fn new(config: AgentConfig, provider: Arc<dyn LLMProvider>) -> Result<Self> {
        let tool_registry = Arc::new(ToolRegistry::new(&config.working_dir));

        Ok(Self {
            config,
            provider,
            tool_registry,
            status: Arc::new(Mutex::new(AgentStatus::Idle)),
            history: Arc::new(Mutex::new(Vec::new())),
            event_tx: None,
        })
    }

    /// Set an event sender for progress reporting
    #[allow(dead_code)]
    pub fn with_event_sender(mut self, tx: mpsc::UnboundedSender<AgentEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Get the current status
    #[allow(dead_code)]
    pub async fn status(&self) -> AgentStatus {
        *self.status.lock().await
    }

    /// Get execution history
    #[allow(dead_code)]
    pub async fn history(&self) -> Vec<AgentStep> {
        self.history.lock().await.clone()
    }

    /// Run the agent with a task
    pub async fn run(&self, task: impl Into<String>) -> Result<AgentResult> {
        let task = task.into();
        let start_time = std::time::Instant::now();

        // Set status to running
        {
            let mut status = self.status.lock().await;
            *status = AgentStatus::Running;
        }

        self.send_event(AgentEvent::Started { task: task.clone() })
            .await;
        self.send_event(AgentEvent::StatusChanged {
            status: AgentStatus::Running,
        })
        .await;

        // Build the system prompt
        let system_prompt = self.build_system_prompt();

        // Initialize conversation
        let mut messages = vec![
            Message {
                role: Role::System,
                content: system_prompt,
                name: None,
            },
            Message {
                role: Role::User,
                content: format!(
                    "Task: {}\n\nPlease complete this task using the available tools. \
                    Think step by step and use tools when needed. \
                    When you're done, provide a summary of what you accomplished.",
                    task
                ),
                name: None,
            },
        ];

        let mut steps = Vec::new();
        let mut final_output = String::new();
        let mut success = false;

        // Main execution loop
        for iteration in 1..=self.config.max_iterations {
            if *self.status.lock().await == AgentStatus::Cancelled {
                break;
            }

            // Get tool definitions
            let tool_definitions = self.tool_registry.get_definitions();

            // Call LLM
            let request = ChatRequest {
                model: self.provider.config().model.clone(),
                messages: messages.clone(),
                temperature: Some(self.config.temperature),
                max_tokens: Some(self.config.max_tokens),
                stream: Some(false),
                tools: if tool_definitions.is_empty() {
                    None
                } else {
                    Some(tool_definitions)
                },
            };

            let response = match self.provider.chat(request).await {
                Ok(r) => r,
                Err(e) => {
                    self.send_event(AgentEvent::Failed {
                        error: format!("LLM error: {}", e),
                    })
                    .await;
                    return Ok(AgentResult {
                        success: false,
                        output: format!("Error: {}", e),
                        steps,
                        total_iterations: iteration,
                        duration_secs: start_time.elapsed().as_secs(),
                    });
                }
            };

            let assistant_message = response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default();

            // Parse tool calls from the response
            let (thought, tool_calls) = self.parse_response(&assistant_message);

            self.send_event(AgentEvent::Thinking {
                iteration,
                thought: thought.clone(),
            })
            .await;

            // Execute tool calls
            let mut tool_results = Vec::new();
            for tool_call in &tool_calls {
                self.send_event(AgentEvent::ToolCalling {
                    iteration,
                    tool_name: tool_call.name.clone(),
                    arguments: tool_call.arguments.clone(),
                })
                .await;

                // Check for shell command confirmation
                if tool_call.name == "shell" && self.config.confirm_shell_commands {
                    let command = tool_call
                        .arguments
                        .get("command")
                        .and_then(|c| c.as_str())
                        .unwrap_or("");

                    if !self.confirm_shell_command(command).await? {
                        tool_results.push(ToolResult::error("User cancelled the command"));
                        continue;
                    }
                }

                let result = self
                    .tool_registry
                    .execute(&tool_call.name, tool_call.arguments.clone())
                    .await;

                self.send_event(AgentEvent::ToolResult {
                    iteration,
                    result: result.clone(),
                })
                .await;

                tool_results.push(result);
            }

            // Record step
            let step = AgentStep {
                iteration,
                thought: thought.clone(),
                tool_calls: tool_calls.clone(),
                tool_results: tool_results.clone(),
                timestamp: chrono::Utc::now(),
            };
            steps.push(step.clone());

            {
                let mut history = self.history.lock().await;
                history.push(step);
            }

            // Check if task is complete
            if tool_calls.is_empty() && self.is_task_complete(&thought) {
                final_output = thought;
                success = true;
                break;
            }

            // Build next message
            let mut next_content = thought;
            if !tool_results.is_empty() {
                next_content.push_str("\n\nTool results:\n");
                for (i, result) in tool_results.iter().enumerate() {
                    next_content.push_str(&format!(
                        "\n[Tool {}]: {}\nSuccess: {}\nOutput: {}\n",
                        i + 1,
                        tool_calls
                            .get(i)
                            .map(|t| t.name.as_str())
                            .unwrap_or("unknown"),
                        result.success,
                        result.output
                    ));
                    if let Some(error) = &result.error {
                        next_content.push_str(&format!("Error: {}\n", error));
                    }
                }
            }

            messages.push(Message {
                role: Role::Assistant,
                content: next_content,
                name: None,
            });

            // Add a continue prompt if no tools were called but task isn't complete
            if tool_calls.is_empty() {
                messages.push(Message {
                    role: Role::User,
                    content: "Continue working on the task. Use tools if needed.".to_string(),
                    name: None,
                });
            }
        }

        // Set final status
        let final_status = if success {
            AgentStatus::Completed
        } else {
            AgentStatus::Failed
        };

        {
            let mut status = self.status.lock().await;
            *status = final_status;
        }

        self.send_event(AgentEvent::StatusChanged {
            status: final_status,
        })
        .await;

        let total_iterations = steps.len() as u32;

        let result = AgentResult {
            success,
            output: final_output,
            steps,
            total_iterations,
            duration_secs: start_time.elapsed().as_secs(),
        };

        self.send_event(AgentEvent::Completed {
            result: result.clone(),
        })
        .await;

        Ok(result)
    }

    /// Cancel the running agent
    #[allow(dead_code)]
    pub async fn cancel(&self) {
        let mut status = self.status.lock().await;
        *status = AgentStatus::Cancelled;
    }

    /// Build the system prompt for the agent
    fn build_system_prompt(&self) -> String {
        let tools = self.tool_registry.list_tools();
        let tool_descriptions: Vec<String> = tools
            .iter()
            .map(|name| {
                let tool = self.tool_registry.get(name).unwrap();
                let def = tool.definition();
                format!(
                    "- {}: {}\n  Parameters: {}",
                    def.name,
                    def.description,
                    serde_json::to_string_pretty(&def.parameters).unwrap_or_default()
                )
            })
            .collect();

        format!(
            "You are an AI agent that can execute tasks autonomously using available tools.\n\n\
            Available tools:\n{}\n\n\
            When you want to use a tool, respond with a JSON object in this format:\n\
            {{{{\n\
            \"thought\": \"Your reasoning here\",\n\
            \"tool_calls\": [\n\
            {{{{\n\
            \"name\": \"tool_name\",\n\
            \"arguments\": {{{{ \"param\": \"value\" }}}}\n\
            }}}}\n\
            ]\n\
            }}}}\n\n\
            If you don't need any tools, just provide your response.\n\
            When the task is complete, clearly indicate this in your response.\n\n\
            Working directory: {}\n\
            Be thorough but efficient. Always check your work.",
            tool_descriptions.join("\n"),
            self.config.working_dir.display()
        )
    }

    /// Parse the agent's response
    fn parse_response(&self, response: &str) -> (String, Vec<ToolCall>) {
        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<Value>(response) {
            let thought = json
                .get("thought")
                .and_then(|t| t.as_str())
                .unwrap_or(response)
                .to_string();

            let tool_calls = json
                .get("tool_calls")
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            let name = item.get("name")?.as_str()?.to_string();
                            let arguments = item.get("arguments").cloned().unwrap_or(Value::Null);
                            Some(ToolCall { name, arguments })
                        })
                        .collect()
                })
                .unwrap_or_default();

            return (thought, tool_calls);
        }

        // Fallback: treat entire response as thought
        (response.to_string(), Vec::new())
    }

    /// Check if the task appears to be complete
    fn is_task_complete(&self, response: &str) -> bool {
        let indicators = [
            "task is complete",
            "i have completed",
            "finished",
            "done",
            "successfully",
            "completed the task",
        ];

        let lower = response.to_lowercase();
        indicators.iter().any(|&ind| lower.contains(ind))
    }

    /// Ask user to confirm a shell command
    async fn confirm_shell_command(&self, command: &str) -> Result<bool> {
        println!("\n🤖 Agent wants to execute shell command:");
        println!("   {}", command);
        println!("\nProceed? [Y/n] ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let response = input.trim().to_lowercase();
        Ok(response.is_empty() || response == "y" || response == "yes")
    }

    /// Send an event if the sender is configured
    async fn send_event(&self, event: AgentEvent) {
        if self.config.verbose {
            match &event {
                AgentEvent::Thinking { iteration, thought } => {
                    println!(
                        "\n🤔 Iteration {}: {}",
                        iteration,
                        thought.lines().next().unwrap_or("")
                    );
                }
                AgentEvent::ToolCalling { tool_name, .. } => {
                    println!("   🔧 Using tool: {}", tool_name);
                }
                AgentEvent::ToolResult { result, .. } => {
                    if result.success {
                        println!("   ✅ Tool succeeded");
                    } else {
                        println!(
                            "   ❌ Tool failed: {}",
                            result.error.as_deref().unwrap_or("unknown error")
                        );
                    }
                }
                _ => {}
            }
        }

        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event);
        }
    }
}

/// Agent command for CLI
pub mod command {
    use super::*;
    use crate::ai::provider::{ProviderConfig, ProviderFactory};
    use clap::Parser;

    /// Agent commands
    #[derive(Debug, Parser, Clone)]
    pub struct AgentCommand {
        /// The task to execute
        #[arg(value_name = "TASK")]
        task: Option<String>,

        /// Maximum iterations
        #[arg(long, default_value = "30")]
        max_iterations: u32,

        /// Working directory
        #[arg(long)]
        working_dir: Option<PathBuf>,

        /// Auto-confirm shell commands (no interactive confirmation)
        #[arg(long)]
        auto_confirm: bool,

        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    }

    impl AgentCommand {
        pub async fn run(&self) -> Result<()> {
            let task = match &self.task {
                Some(t) => t.clone(),
                None => {
                    println!("🤖 Arb Agent Mode");
                    println!("Enter your task (press Enter twice to submit):\n");

                    let mut lines = Vec::new();
                    let stdin = tokio::io::stdin();
                    let reader = tokio::io::BufReader::new(stdin);
                    use tokio::io::AsyncBufReadExt;

                    let mut lines_stream = reader.lines();
                    let mut empty_line_count = 0;

                    while let Ok(Some(line)) = lines_stream.next_line().await {
                        if line.trim().is_empty() {
                            empty_line_count += 1;
                            if empty_line_count >= 1 && !lines.is_empty() {
                                break;
                            }
                        } else {
                            empty_line_count = 0;
                            lines.push(line);
                        }
                    }

                    if lines.is_empty() {
                        println!("No task provided. Exiting.");
                        return Ok(());
                    }

                    lines.join("\n")
                }
            };

            // Load configuration
            let ai_config = super::super::command::load_ai_config().await?;
            let provider_config = ProviderConfig {
                name: ai_config.name,
                api_url: ai_config.api_url,
                api_key: super::super::command::get_api_key().await?,
                model: ai_config.model,
                timeout_seconds: 60,
                temperature: ai_config.temperature,
                max_tokens: ai_config.max_tokens,
                headers: vec![],
            };

            let provider = ProviderFactory::create(provider_config)?;

            // Create agent configuration
            let agent_config = AgentConfig {
                max_iterations: self.max_iterations,
                max_tokens: 4096,
                temperature: 0.7,
                working_dir: self.working_dir.clone().unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                }),
                confirm_shell_commands: !self.auto_confirm,
                verbose: self.verbose,
            };

            println!("\n🚀 Starting agent...\n");

            // Create and run agent
            let provider: Arc<dyn LLMProvider> = Arc::from(provider);
            let agent = Agent::new(agent_config, provider)?;
            let result = agent.run(task).await?;

            // Print results
            println!("\n{}\n", "═".repeat(60));
            if result.success {
                println!("✅ Task completed successfully!");
            } else {
                println!("❌ Task failed or was cancelled");
            }
            println!("📊 Statistics:");
            println!("   Iterations: {}", result.total_iterations);
            println!("   Duration: {}s", result.duration_secs);
            println!("\n📄 Output:\n{}", result.output);

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.max_iterations, 30);
        assert_eq!(config.max_tokens, 4096);
        assert!(config.confirm_shell_commands);
    }

    #[test]
    fn test_agent_status_serialization() {
        let status = AgentStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"Running\"");
    }
}
