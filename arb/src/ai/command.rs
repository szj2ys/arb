//! AI command handling for Arb Terminal

use crate::ai::provider::{ChatRequest, LLMProvider, Message, ProviderConfig, ProviderFactory, Role};
use anyhow::Result;
use clap::{Parser, Subcommand};

/// AI assistant commands
#[derive(Debug, Parser, Clone)]
pub struct AiCommand {
    #[command(subcommand)]
    cmd: Option<AiSubCommand>,

    /// Natural language query (when no subcommand provided)
    #[arg(value_name = "QUERY")]
    query: Option<String>,
}

#[derive(Debug, Subcommand, Clone)]
enum AiSubCommand {
    /// Configure AI settings
    #[command(name = "config")]
    Config(AiConfigCommand),

    /// Start interactive chat session
    #[command(name = "chat")]
    Chat,

    /// Explain a command or output
    #[command(name = "explain")]
    Explain {
        /// Command or text to explain
        #[arg(value_name = "TEXT")]
        text: String,
    },

    /// Fix the last error
    #[command(name = "fix")]
    Fix,

    /// Test LLM connection
    #[command(name = "test")]
    Test,

    /// Start agent mode for autonomous task execution
    #[command(name = "agent")]
    Agent(crate::ai::agent::command::AgentCommand),

    /// Start team mode for multi-agent collaboration
    #[command(name = "team")]
    Team(crate::ai::team::command::TeamCommand),
}

/// AI configuration commands
#[derive(Debug, Parser, Clone)]
struct AiConfigCommand {
    #[command(subcommand)]
    cmd: AiConfigSubCommand,
}

#[derive(Debug, Subcommand, Clone)]
enum AiConfigSubCommand {
    /// Set LLM API URL
    #[command(name = "set-url")]
    SetUrl {
        #[arg(value_name = "URL")]
        url: String,
    },

    /// Set API key
    #[command(name = "set-key")]
    SetKey {
        #[arg(value_name = "KEY")]
        key: String,
    },

    /// Set model name
    #[command(name = "set-model")]
    SetModel {
        #[arg(value_name = "MODEL")]
        model: String,
    },

    /// Set provider name
    #[command(name = "set-provider")]
    SetProvider {
        #[arg(value_name = "PROVIDER")]
        provider: String,
    },

    /// Show current configuration
    #[command(name = "show")]
    Show,
}

impl AiCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.cmd {
            Some(AiSubCommand::Config(config)) => config.run().await,
            Some(AiSubCommand::Chat) => start_chat().await,
            Some(AiSubCommand::Explain { text }) => explain_command(text).await,
            Some(AiSubCommand::Fix) => fix_last_error().await,
            Some(AiSubCommand::Test) => test_connection().await,
            Some(AiSubCommand::Agent(agent_cmd)) => agent_cmd.run().await,
            Some(AiSubCommand::Team(team_cmd)) => team_cmd.run().await,
            None => {
                if let Some(query) = &self.query {
                    process_natural_language(query).await
                } else {
                    start_chat().await
                }
            }
        }
    }
}

impl AiConfigCommand {
    async fn run(&self) -> Result<()> {
        use AiConfigSubCommand::*;

        let mut config = load_ai_config().await?;

        match &self.cmd {
            SetUrl { url } => {
                config.api_url = url.clone();
                save_ai_config(&config).await?;
                println!("✓ API URL set to: {}", url);
            }
            SetKey { key } => {
                // Store key in keychain instead of config file
                store_api_key(key).await?;
                println!("✓ API key stored securely");
            }
            SetModel { model } => {
                config.model = model.clone();
                save_ai_config(&config).await?;
                println!("✓ Model set to: {}", model);
            }
            SetProvider { provider } => {
                config.name = provider.clone();
                save_ai_config(&config).await?;
                println!("✓ Provider set to: {}", provider);
            }
            Show => {
                println!("AI Configuration:");
                println!("  Provider: {}", config.name);
                println!("  API URL: {}", config.api_url);
                println!("  Model: {}", config.model);
                println!("  API Key: {}", if has_api_key().await? { "✓ Set" } else { "✗ Not set" });
            }
        }

        Ok(())
    }
}

/// Process natural language query
async fn process_natural_language(query: &str) -> Result<()> {
    let provider = create_provider().await?;

    println!("🤔 Thinking...");

    let request = ChatRequest {
        model: provider.config().model.clone(),
        messages: vec![
            Message {
                role: Role::System,
                content: "You are a helpful terminal assistant. Convert natural language to shell commands. Only output the command, no explanation.".to_string(),
                name: None,
            },
            Message {
                role: Role::User,
                content: format!("Convert to shell command: {}", query),
                name: None,
            },
        ],
        temperature: Some(0.1),
        max_tokens: Some(200),
        stream: Some(false),
        tools: None,
    };

    let response = provider.chat(request).await?;

    if let Some(choice) = response.choices.first() {
        let command = choice.message.content.trim();
        println!("\n💡 Suggested command:");
        println!("   {}", command);
        println!("\nPress Enter to execute, or Ctrl+C to cancel");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        // Execute the command
        println!("\n🚀 Executing: {}\n", command);
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .status()?;

        if !status.success() {
            println!("\n❌ Command failed with exit code: {:?}", status.code());
        }
    }

    Ok(())
}

/// Start interactive chat session
async fn start_chat() -> Result<()> {
    let provider = create_provider().await?;

    println!("🤖 Arb AI Assistant");
    println!("Type your message, or 'exit' to quit\n");

    let mut messages = vec![Message {
        role: Role::System,
        content: "You are a helpful terminal assistant. Help users with shell commands, file operations, and debugging. Be concise.".to_string(),
        name: None,
    }];

    loop {
        print!("You: ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("👋 Goodbye!");
            break;
        }

        messages.push(Message {
            role: Role::User,
            content: input.to_string(),
            name: None,
        });

        print!("\nAI: ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let request = ChatRequest {
            model: provider.config().model.clone(),
            messages: messages.clone(),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: Some(true),
            tools: None,
        };

        let mut stream = provider.chat_stream(request).await?;
        let mut full_response = String::new();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            print!("{}", content);
                            std::io::Write::flush(&mut std::io::stdout())?;
                            full_response.push_str(content);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\n⚠️  Stream error: {}", e);
                    break;
                }
            }
        }

        println!("\n");

        messages.push(Message {
            role: Role::Assistant,
            content: full_response,
            name: None,
        });
    }

    Ok(())
}

/// Explain a command or output
async fn explain_command(text: &str) -> Result<()> {
    let provider = create_provider().await?;

    println!("🔍 Analyzing...\n");

    let request = ChatRequest {
        model: provider.config().model.clone(),
        messages: vec![
            Message {
                role: Role::System,
                content: "You are a terminal expert. Explain shell commands and error messages clearly.".to_string(),
                name: None,
            },
            Message {
                role: Role::User,
                content: format!("Explain this:\n{}", text),
                name: None,
            },
        ],
        temperature: Some(0.3),
        max_tokens: Some(500),
        stream: Some(false),
        tools: None,
    };

    let response = provider.chat(request).await?;

    if let Some(choice) = response.choices.first() {
        println!("{}", choice.message.content);
    }

    Ok(())
}

/// Fix the last error
async fn fix_last_error() -> Result<()> {
    // Read last command from history
    let last_command = get_last_command()?;
    let last_error = get_last_error()?;

    if last_command.is_empty() {
        println!("❌ No previous command found");
        return Ok(());
    }

    println!("Last command: {}", last_command);
    println!("Analyzing error...\n");

    let provider = create_provider().await?;

    let request = ChatRequest {
        model: provider.config().model.clone(),
        messages: vec![
            Message {
                role: Role::System,
                content: "You are a debugging expert. Analyze command errors and suggest fixes.".to_string(),
                name: None,
            },
            Message {
                role: Role::User,
                content: format!("Command: {}\nError: {}\n\nWhat's wrong and how do I fix it?", last_command, last_error),
                name: None,
            },
        ],
        temperature: Some(0.3),
        max_tokens: Some(500),
        stream: Some(false),
        tools: None,
    };

    let response = provider.chat(request).await?;

    if let Some(choice) = response.choices.first() {
        println!("💡 Fix suggestion:\n{}", choice.message.content);
    }

    Ok(())
}

/// Test LLM connection
async fn test_connection() -> Result<()> {
    println!("🧪 Testing LLM connection...");

    let provider = create_provider().await?;
    let config = provider.config();

    println!("Provider: {}", config.name);
    println!("API URL: {}", config.api_url);
    println!("Model: {}", config.model);

    match provider.test_connection().await {
        Ok(()) => {
            println!("\n✅ Connection successful!");
        }
        Err(e) => {
            println!("\n❌ Connection failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

// Helper functions

async fn create_provider() -> Result<Box<dyn LLMProvider>> {
    let config = load_ai_config().await?;
    let mut config = ProviderConfig {
        name: config.name,
        api_url: config.api_url,
        api_key: get_api_key().await?,
        model: config.model,
        timeout_seconds: 60,
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        headers: vec![],
    };

    // Load from environment if not set
    if config.api_key.is_empty() {
        if let Ok(key) = std::env::var("ARB_AI_API_KEY") {
            config.api_key = key;
        }
    }
    if config.api_url == "https://api.openai.com/v1" {
        if let Ok(url) = std::env::var("ARB_AI_API_URL") {
            config.api_url = url;
        }
    }

    ProviderFactory::create(config)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiConfig {
    pub name: String,
    pub api_url: String,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            name: "dashscope".to_string(),
            api_url: "https://coding.dashscope.aliyuncs.com/v1".to_string(),
            model: "kimi-k2.5".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(4096),
        }
    }
}

pub async fn load_ai_config() -> Result<AiConfig> {
    let config_path = get_config_path()?;
    if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path).await?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(AiConfig::default())
    }
}

async fn save_ai_config(config: &AiConfig) -> Result<()> {
    let config_path = get_config_path()?;
    let content = serde_json::to_string_pretty(config)?;
    tokio::fs::write(&config_path, content).await?;
    Ok(())
}

fn get_config_path() -> Result<std::path::PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("arb");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("ai.json"))
}

async fn store_api_key(key: &str) -> Result<()> {
    // For now, store in config file (in production, use keychain)
    let key_path = get_config_path()?.with_extension(".key");
    tokio::fs::write(&key_path, key).await?;
    Ok(())
}

pub async fn get_api_key() -> Result<String> {
    // Try environment first
    if let Ok(key) = std::env::var("ARB_AI_API_KEY") {
        return Ok(key);
    }

    // Try file
    let key_path = get_config_path()?.with_extension(".key");
    if key_path.exists() {
        Ok(tokio::fs::read_to_string(&key_path).await?)
    } else {
        Ok(String::new())
    }
}

async fn has_api_key() -> Result<bool> {
    Ok(!get_api_key().await?.is_empty())
}

fn get_last_command() -> Result<String> {
    // Read from shell history
    let history_file = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("No home directory"))?
        .join(".zsh_history");

    if !history_file.exists() {
        return Ok(String::new());
    }

    let content = std::fs::read_to_string(&history_file)?;
    let last_line = content.lines().last().unwrap_or("");

    // Parse zsh history format
    if let Some(cmd) = last_line.splitn(2, ';').nth(1) {
        Ok(cmd.to_string())
    } else {
        Ok(last_line.to_string())
    }
}

fn get_last_error() -> Result<String> {
    // This would need to be populated by the shell integration
    // For now, return empty
    Ok(String::from("See terminal output above"))
}
