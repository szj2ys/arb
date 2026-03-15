//! Team Agent - Multi-agent collaboration system
//!
//! Provides a message bus and orchestration for multiple specialized agents
//! working together on complex tasks.

use crate::ai::agent::{Agent, AgentConfig, AgentResult, AgentStatus};
use crate::ai::provider::{LLMProvider, ProviderConfig, ProviderFactory};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Team configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    /// Maximum number of parallel agents
    pub max_parallel_agents: usize,
    /// Message timeout in seconds
    pub message_timeout_secs: u64,
    /// Whether agents share context
    pub shared_context: bool,
    /// Working directory for the team
    pub working_dir: PathBuf,
}

impl Default for TeamConfig {
    fn default() -> Self {
        Self {
            max_parallel_agents: 4,
            message_timeout_secs: 300,
            shared_context: true,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

/// Agent role/specialization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    /// Code reviewer - reviews code for quality and issues
    CodeReviewer,
    /// Architect - designs and reviews architecture
    Architect,
    /// Tester - writes and runs tests
    Tester,
    /// Researcher - explores and researches solutions
    Researcher,
    /// Implementer - implements features
    Implementer,
    /// Debugger - debugs and fixes issues
    Debugger,
    /// Custom role
    Custom(String),
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentRole::CodeReviewer => write!(f, "code-reviewer"),
            AgentRole::Architect => write!(f, "architect"),
            AgentRole::Tester => write!(f, "tester"),
            AgentRole::Researcher => write!(f, "researcher"),
            AgentRole::Implementer => write!(f, "implementer"),
            AgentRole::Debugger => write!(f, "debugger"),
            AgentRole::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Team member (agent instance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    /// Unique identifier
    pub id: String,
    /// Agent role
    pub role: AgentRole,
    /// Display name
    pub name: String,
    /// Current status
    pub status: AgentStatus,
    /// Current task
    pub current_task: Option<String>,
    /// Model configuration
    pub model: String,
}

/// Message sent between team members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMessage {
    /// Message ID
    pub id: String,
    /// Sender member ID
    pub from: String,
    /// Recipient member ID (None for broadcast)
    pub to: Option<String>,
    /// Message type
    pub message_type: MessageType,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Task assignment
    TaskAssigned,
    /// Task completion
    TaskCompleted,
    /// Status update
    StatusUpdate,
    /// Question
    Question,
    /// Answer
    Answer,
    /// Broadcast to all
    Broadcast,
    /// Result sharing
    Result,
    /// Error
    Error,
}

/// Team execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamResult {
    /// Overall success
    pub success: bool,
    /// Results from each member
    pub member_results: HashMap<String, AgentResult>,
    /// Messages exchanged
    pub messages: Vec<TeamMessage>,
    /// Duration in seconds
    pub duration_secs: u64,
}

/// Shared context for team collaboration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SharedContext {
    /// Key-value store
    pub data: HashMap<String, serde_json::Value>,
    /// Files being worked on
    pub active_files: Vec<String>,
    /// Decisions made
    pub decisions: Vec<String>,
}

/// Team event for progress reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeamEvent {
    /// Team started
    Started { mission: String },
    /// Member joined
    MemberJoined { member: TeamMember },
    /// Member left
    MemberLeft { member_id: String },
    /// Message sent
    MessageSent { message: TeamMessage },
    /// Member status changed
    MemberStatusChanged {
        member_id: String,
        status: AgentStatus,
    },
    /// Task assigned to member
    TaskAssigned { member_id: String, task: String },
    /// Task completed by member
    TaskCompleted {
        member_id: String,
        result: AgentResult,
    },
    /// Team completed
    Completed { result: TeamResult },
    /// Team failed
    Failed { error: String },
}

/// The Team manages multiple agents working together
#[allow(dead_code)]
pub struct Team {
    config: TeamConfig,
    #[allow(clippy::type_complexity)]
    members: Arc<RwLock<HashMap<String, (TeamMember, Arc<dyn LLMProvider>)>>>,
    message_bus: Arc<RwLock<Vec<TeamMessage>>>,
    shared_context: Arc<RwLock<SharedContext>>,
    event_tx: Option<mpsc::UnboundedSender<TeamEvent>>,
    running: Arc<RwLock<bool>>,
}

impl Team {
    /// Create a new team with the given configuration
    pub fn new(config: TeamConfig) -> Self {
        Self {
            config,
            members: Arc::new(RwLock::new(HashMap::new())),
            message_bus: Arc::new(RwLock::new(Vec::new())),
            shared_context: Arc::new(RwLock::new(SharedContext::default())),
            event_tx: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Set an event sender for progress reporting
    #[allow(dead_code)]
    pub fn with_event_sender(mut self, tx: mpsc::UnboundedSender<TeamEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Add a member to the team
    pub async fn add_member(
        &self,
        role: AgentRole,
        name: impl Into<String>,
        model: impl Into<String>,
        provider_config: Option<ProviderConfig>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let name = name.into();
        let model = model.into();

        // Create provider
        let provider_config = provider_config.unwrap_or_else(|| ProviderConfig {
            name: "dashscope".to_string(),
            api_url: "https://coding.dashscope.aliyuncs.com/v1".to_string(),
            api_key: String::new(),
            model: model.clone(),
            timeout_seconds: 60,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            headers: vec![],
        });

        let provider: Arc<dyn LLMProvider> = Arc::from(ProviderFactory::create(provider_config)?);

        let member = TeamMember {
            id: id.clone(),
            role: role.clone(),
            name: name.clone(),
            status: AgentStatus::Idle,
            current_task: None,
            model: model.clone(),
        };

        {
            let mut members = self.members.write().await;
            members.insert(id.clone(), (member.clone(), provider));
        }

        self.send_event(TeamEvent::MemberJoined { member }).await;

        Ok(id)
    }

    /// Remove a member from the team
    #[allow(dead_code)]
    pub async fn remove_member(&self, member_id: &str) -> Result<()> {
        {
            let mut members = self.members.write().await;
            members.remove(member_id);
        }

        self.send_event(TeamEvent::MemberLeft {
            member_id: member_id.to_string(),
        })
        .await;

        Ok(())
    }

    /// Get all team members
    pub async fn get_members(&self) -> Vec<TeamMember> {
        let members = self.members.read().await;
        members.values().map(|(m, _)| m.clone()).collect()
    }

    /// Send a message between members
    #[allow(dead_code)]
    pub async fn send_message(
        &self,
        from: impl Into<String>,
        to: Option<impl Into<String>>,
        message_type: MessageType,
        content: impl Into<String>,
    ) -> Result<String> {
        let message = TeamMessage {
            id: Uuid::new_v4().to_string(),
            from: from.into(),
            to: to.map(|t| t.into()),
            message_type,
            content: content.into(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        };

        {
            let mut bus = self.message_bus.write().await;
            bus.push(message.clone());
        }

        self.send_event(TeamEvent::MessageSent {
            message: message.clone(),
        })
        .await;

        Ok(message.id)
    }

    /// Broadcast a message to all members
    #[allow(dead_code)]
    pub async fn broadcast(
        &self,
        from: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<String> {
        self.send_message(from, None::<String>, MessageType::Broadcast, content)
            .await
    }

    /// Assign a task to a specific member
    pub async fn assign_task(
        &self,
        member_id: impl Into<String>,
        task: impl Into<String>,
    ) -> Result<AgentResult> {
        let member_id = member_id.into();
        let task = task.into();

        // Update member status
        {
            let mut members = self.members.write().await;
            if let Some((member, _)) = members.get_mut(&member_id) {
                member.status = AgentStatus::Running;
                member.current_task = Some(task.clone());
            }
        }

        self.send_event(TeamEvent::TaskAssigned {
            member_id: member_id.clone(),
            task: task.clone(),
        })
        .await;

        self.send_event(TeamEvent::MemberStatusChanged {
            member_id: member_id.clone(),
            status: AgentStatus::Running,
        })
        .await;

        // Get member and provider
        let (member, provider) = {
            let members = self.members.read().await;
            match members.get(&member_id) {
                Some((m, p)) => (m.clone(), Arc::clone(p)),
                None => return Err(anyhow!("Member not found: {}", member_id)),
            }
        };

        // Create agent configuration with role-specific system prompt
        let agent_config = AgentConfig {
            max_iterations: 30,
            max_tokens: 4096,
            temperature: 0.7,
            working_dir: self.config.working_dir.clone(),
            confirm_shell_commands: false,
            verbose: false,
        };

        // Build task with role context
        let contextualized_task = self.build_role_task(&member.role, &task);

        // Create and run agent
        let agent = Agent::new(agent_config, provider)?;
        let result = agent.run(contextualized_task).await?;

        // Update member status
        {
            let mut members = self.members.write().await;
            if let Some((member, _)) = members.get_mut(&member_id) {
                member.status = AgentStatus::Completed;
                member.current_task = None;
            }
        }

        self.send_event(TeamEvent::TaskCompleted {
            member_id: member_id.clone(),
            result: result.clone(),
        })
        .await;

        self.send_event(TeamEvent::MemberStatusChanged {
            member_id,
            status: AgentStatus::Completed,
        })
        .await;

        Ok(result)
    }

    /// Run the team with a mission
    pub async fn run(&self, mission: impl Into<String>) -> Result<TeamResult> {
        let mission = mission.into();
        let start_time = std::time::Instant::now();

        *self.running.write().await = true;

        self.send_event(TeamEvent::Started {
            mission: mission.clone(),
        })
        .await;

        // Get all members
        let members = self.get_members().await;

        if members.is_empty() {
            return Err(anyhow!("No team members available"));
        }

        // Analyze mission and delegate tasks to appropriate members
        let tasks = self.analyze_mission(&mission, &members).await?;

        // Execute tasks in parallel (up to max_parallel_agents)
        use futures::stream::{self, StreamExt};
        let max_parallel = self.config.max_parallel_agents;

        let member_results: HashMap<String, AgentResult> = stream::iter(tasks)
            .map(|(member_id, task)| async move {
                let result = self.assign_task(member_id.clone(), task).await;
                (member_id, result)
            })
            .buffer_unordered(max_parallel)
            .filter_map(|(id, result)| async move {
                match result {
                    Ok(r) => Some((id, r)),
                    Err(e) => {
                        eprintln!("Task failed: {}", e);
                        None
                    }
                }
            })
            .collect()
            .await;

        *self.running.write().await = false;

        let messages = self.message_bus.read().await.clone();

        let success = member_results.values().all(|r| r.success);

        let result = TeamResult {
            success,
            member_results,
            messages,
            duration_secs: start_time.elapsed().as_secs(),
        };

        if success {
            self.send_event(TeamEvent::Completed {
                result: result.clone(),
            })
            .await;
        } else {
            self.send_event(TeamEvent::Failed {
                error: "Some tasks failed".to_string(),
            })
            .await;
        }

        Ok(result)
    }

    /// Stop the team
    #[allow(dead_code)]
    pub async fn stop(&self) {
        *self.running.write().await = false;
    }

    /// Check if team is running
    #[allow(dead_code)]
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Build task with role-specific context
    fn build_role_task(&self, role: &AgentRole, task: &str) -> String {
        let role_prompt = match role {
            AgentRole::CodeReviewer => {
                "You are a code reviewer. Focus on: code quality, potential bugs, security issues, \
                performance concerns, and adherence to best practices. Be thorough and constructive."
            }
            AgentRole::Architect => {
                "You are a software architect. Focus on: system design, patterns, trade-offs, \
                scalability, and maintainability. Think about long-term implications."
            }
            AgentRole::Tester => {
                "You are a QA engineer. Focus on: test coverage, edge cases, test strategies, \
                and quality assurance. Find bugs before they reach production."
            }
            AgentRole::Researcher => {
                "You are a researcher. Explore solutions, evaluate alternatives, \
                and provide comprehensive analysis of options."
            }
            AgentRole::Implementer => {
                "You are an implementer. Write clean, working code that follows best practices. \
                Focus on getting things done correctly."
            }
            AgentRole::Debugger => {
                "You are a debugger. Find root causes of issues, analyze logs and errors, \
                and propose targeted fixes."
            }
            AgentRole::Custom(_) => {
                "You are a team member. Contribute your expertise to achieve the team goal."
            }
        };

        format!("{}\n\nTask: {}", role_prompt, task)
    }

    /// Analyze mission and create task assignments
    async fn analyze_mission(
        &self,
        mission: &str,
        members: &[TeamMember],
    ) -> Result<Vec<(String, String)>> {
        // Simple delegation: assign parts of the mission to appropriate roles
        let mut tasks = Vec::new();

        // Break down mission based on available roles
        for member in members {
            let task = match &member.role {
                AgentRole::CodeReviewer => {
                    format!("Review the codebase for issues related to: {}", mission)
                }
                AgentRole::Architect => {
                    format!("Design the architecture for: {}", mission)
                }
                AgentRole::Tester => {
                    format!("Create test plan and identify test cases for: {}", mission)
                }
                AgentRole::Researcher => {
                    format!("Research and evaluate approaches for: {}", mission)
                }
                AgentRole::Implementer => {
                    format!("Implement the solution for: {}", mission)
                }
                AgentRole::Debugger => {
                    format!("Debug and fix issues related to: {}", mission)
                }
                AgentRole::Custom(role_name) => {
                    format!("As the {}, contribute to: {}", role_name, mission)
                }
            };

            tasks.push((member.id.clone(), task));
        }

        Ok(tasks)
    }

    /// Send an event if the sender is configured
    async fn send_event(&self, event: TeamEvent) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event);
        }
    }
}

/// Team command for CLI
pub mod command {
    use super::*;
    use clap::{Parser, Subcommand};

    /// Team agent commands
    #[derive(Debug, Parser, Clone)]
    pub struct TeamCommand {
        #[command(subcommand)]
        cmd: Option<TeamSubCommand>,

        /// The mission to execute (when no subcommand provided)
        #[arg(value_name = "MISSION")]
        mission: Option<String>,
    }

    #[derive(Debug, Subcommand, Clone)]
    enum TeamSubCommand {
        /// Start a team session
        #[command(name = "start")]
        Start,

        /// Add a member to the team
        #[command(name = "add")]
        Add {
            /// Role of the member
            #[arg(value_name = "ROLE")]
            role: String,

            /// Name of the member
            #[arg(value_name = "NAME")]
            name: Option<String>,
        },

        /// Remove a member from the team
        #[command(name = "remove")]
        Remove {
            /// Member ID
            #[arg(value_name = "ID")]
            id: String,
        },

        /// List team members
        #[command(name = "list")]
        List,

        /// Assign a task to a member
        #[command(name = "assign")]
        Assign {
            /// Member ID
            #[arg(value_name = "ID")]
            member_id: String,

            /// Task description
            #[arg(value_name = "TASK")]
            task: String,
        },

        /// Broadcast a message to all members
        #[command(name = "broadcast")]
        Broadcast {
            /// Message content
            #[arg(value_name = "MESSAGE")]
            message: String,
        },

        /// Show team status
        #[command(name = "status")]
        Status,

        /// Stop the team
        #[command(name = "stop")]
        Stop,
    }

    impl TeamCommand {
        pub async fn run(&self) -> Result<()> {
            match &self.cmd {
                Some(TeamSubCommand::Start) => {
                    println!("🚀 Starting team session...");
                    println!("Use 'arb team add <role>' to add members");
                    Ok(())
                }
                Some(TeamSubCommand::Add { role, name }) => {
                    let role = parse_role(role);
                    let name = name.clone().unwrap_or_else(|| role.to_string());
                    println!("👤 Adding team member: {} ({})", name, role);
                    // TODO: Implement team session management
                    Ok(())
                }
                Some(TeamSubCommand::Remove { id }) => {
                    println!("🗑️  Removing team member: {}", id);
                    Ok(())
                }
                Some(TeamSubCommand::List) => {
                    println!("📋 Team members:");
                    println!("   (No active team session)");
                    Ok(())
                }
                Some(TeamSubCommand::Assign { member_id, task }) => {
                    println!("📋 Assigning task to {}: {}", member_id, task);
                    Ok(())
                }
                Some(TeamSubCommand::Broadcast { message }) => {
                    println!("📢 Broadcasting: {}", message);
                    Ok(())
                }
                Some(TeamSubCommand::Status) => {
                    println!("📊 Team status: No active session");
                    Ok(())
                }
                Some(TeamSubCommand::Stop) => {
                    println!("🛑 Stopping team session");
                    Ok(())
                }
                None => {
                    if let Some(mission) = &self.mission {
                        // Quick team execution with default members
                        println!("🚀 Running team mission: {}", mission);

                        let config = TeamConfig::default();
                        let team = Team::new(config);

                        // Add default team members
                        let _architect = team
                            .add_member(AgentRole::Architect, "Architect", "kimi-k2.5", None)
                            .await?;

                        let _implementer = team
                            .add_member(AgentRole::Implementer, "Implementer", "kimi-k2.5", None)
                            .await?;

                        let _reviewer = team
                            .add_member(AgentRole::CodeReviewer, "Reviewer", "kimi-k2.5", None)
                            .await?;

                        // Run the mission
                        let result = team.run(mission).await?;

                        // Display results
                        println!("\n{}\n", "═".repeat(60));
                        if result.success {
                            println!("✅ Mission completed successfully!");
                        } else {
                            println!("⚠️  Mission completed with some issues");
                        }
                        println!("📊 Statistics:");
                        println!("   Duration: {}s", result.duration_secs);
                        println!("   Messages exchanged: {}", result.messages.len());
                        println!("   Members contributed: {}", result.member_results.len());

                        for (id, member_result) in &result.member_results {
                            println!("\n👤 Member: {}", id);
                            println!("   Success: {}", member_result.success);
                            println!("   Iterations: {}", member_result.total_iterations);
                            if !member_result.output.is_empty() {
                                println!(
                                    "   Output preview: {}",
                                    member_result
                                        .output
                                        .lines()
                                        .next()
                                        .unwrap_or("")
                                        .chars()
                                        .take(100)
                                        .collect::<String>()
                                );
                            }
                        }

                        Ok(())
                    } else {
                        println!("🤖 Arb Team Agent");
                        println!();
                        println!("Usage:");
                        println!("  arb team \"<mission>\"       Run a team mission");
                        println!("  arb team start              Start a team session");
                        println!("  arb team add <role>         Add a team member");
                        println!("  arb team list               List team members");
                        println!("  arb team status             Show team status");
                        println!();
                        println!("Available roles:");
                        println!("  architect      System architect");
                        println!("  implementer    Code implementer");
                        println!("  reviewer       Code reviewer");
                        println!("  tester         QA tester");
                        println!("  researcher     Researcher");
                        println!("  debugger       Debugger");
                        Ok(())
                    }
                }
            }
        }
    }

    fn parse_role(role: &str) -> AgentRole {
        match role.to_lowercase().as_str() {
            "architect" => AgentRole::Architect,
            "implementer" | "implementor" => AgentRole::Implementer,
            "reviewer" | "code-reviewer" => AgentRole::CodeReviewer,
            "tester" | "qa" => AgentRole::Tester,
            "researcher" => AgentRole::Researcher,
            "debugger" => AgentRole::Debugger,
            _ => AgentRole::Custom(role.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_config_default() {
        let config = TeamConfig::default();
        assert_eq!(config.max_parallel_agents, 4);
        assert_eq!(config.message_timeout_secs, 300);
        assert!(config.shared_context);
    }

    #[test]
    fn test_agent_role_display() {
        assert_eq!(AgentRole::Architect.to_string(), "architect");
        assert_eq!(AgentRole::CodeReviewer.to_string(), "code-reviewer");
        assert_eq!(
            AgentRole::Custom("specialist".to_string()).to_string(),
            "specialist"
        );
    }

    #[test]
    fn test_team_message_creation() {
        let msg = TeamMessage {
            id: "test-id".to_string(),
            from: "agent1".to_string(),
            to: Some("agent2".to_string()),
            message_type: MessageType::TaskAssigned,
            content: "Do this task".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        };

        assert_eq!(msg.from, "agent1");
        assert_eq!(msg.to, Some("agent2".to_string()));
    }
}
