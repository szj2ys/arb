//! Claude Code Integration
//!
//! Reuses Claude Code configuration:
//! - Skills from `.claude/skills/`
//! - Commands from `.claude/commands/`
//! - Subagents from `.claude/agents/`

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Claude Code configuration directory
pub const CLAUDE_DIR: &str = ".claude";

/// Skill definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub file_path: PathBuf,
}

/// Command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub file_path: PathBuf,
}

/// Subagent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subagent {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub tools: Vec<String>,
    pub file_path: PathBuf,
}

/// Loaded Claude Code configuration
#[derive(Debug, Clone, Default)]
pub struct ClaudeConfig {
    pub skills: HashMap<String, Skill>,
    pub commands: HashMap<String, Command>,
    pub subagents: HashMap<String, Subagent>,
}

impl ClaudeConfig {
    /// Load Claude Code configuration from the given directory
    pub fn load(project_dir: impl AsRef<Path>) -> Result<Self> {
        let project_dir = project_dir.as_ref();
        let claude_dir = project_dir.join(CLAUDE_DIR);

        if !claude_dir.exists() {
            return Ok(Self::default());
        }

        let mut config = Self::default();

        // Load skills
        let skills_dir = claude_dir.join("skills");
        if skills_dir.exists() {
            config.load_skills(&skills_dir)?;
        }

        // Load commands
        let commands_dir = claude_dir.join("commands");
        if commands_dir.exists() {
            config.load_commands(&commands_dir)?;
        }

        // Load subagents
        let agents_dir = claude_dir.join("agents");
        if agents_dir.exists() {
            config.load_subagents(&agents_dir)?;
        }

        Ok(config)
    }

    fn load_skills(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let content = std::fs::read_to_string(&path)?;
                let description = extract_description(&content);

                self.skills.insert(
                    name.clone(),
                    Skill {
                        name,
                        description,
                        content,
                        file_path: path,
                    },
                );
            }
        }
        Ok(())
    }

    fn load_commands(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let content = std::fs::read_to_string(&path)?;
                let (description, prompt) = parse_command_file(&content);

                self.commands.insert(
                    name.clone(),
                    Command {
                        name,
                        description,
                        prompt,
                        file_path: path,
                    },
                );
            }
        }
        Ok(())
    }

    fn load_subagents(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let content = std::fs::read_to_string(&path)?;
                let (description, prompt, tools) = parse_agent_file(&content);

                self.subagents.insert(
                    name.clone(),
                    Subagent {
                        name,
                        description,
                        prompt,
                        tools,
                        file_path: path,
                    },
                );
            }
        }
        Ok(())
    }

    /// Get a skill by name
    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// Get a command by name
    pub fn get_command(&self, name: &str) -> Option<&Command> {
        self.commands.get(name)
    }

    /// Get a subagent by name
    pub fn get_subagent(&self, name: &str) -> Option<&Subagent> {
        self.subagents.get(name)
    }

    /// Check if any Claude Code configuration exists
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty() && self.commands.is_empty() && self.subagents.is_empty()
    }

    /// Get all skill names
    pub fn skill_names(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Get all command names
    pub fn command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    /// Get all subagent names
    pub fn subagent_names(&self) -> Vec<&str> {
        self.subagents.keys().map(|s| s.as_str()).collect()
    }
}

/// Extract description from skill content (first line or first paragraph)
fn extract_description(content: &str) -> String {
    content
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().trim_start_matches("# ").trim_start_matches("// ").to_string())
        .unwrap_or_else(|| "No description".to_string())
}

/// Parse a command file (markdown with frontmatter)
fn parse_command_file(content: &str) -> (String, String) {
    let mut description = "No description".to_string();
    let mut prompt = content.to_string();

    // Check for YAML frontmatter
    if content.starts_with("---") {
        if let Some(end) = content.find("\n---\n") {
            let frontmatter = &content[4..end];
            prompt = content[end + 5..].to_string();

            // Simple YAML parsing for description
            for line in frontmatter.lines() {
                if line.starts_with("description:") {
                    description = line["description:".len()..].trim().to_string();
                    break;
                }
            }
        }
    }

    (description, prompt)
}

/// Parse an agent file (markdown with frontmatter)
fn parse_agent_file(content: &str) -> (String, String, Vec<String>) {
    let mut description = "No description".to_string();
    let mut tools = Vec::new();
    let mut prompt = content.to_string();

    // Check for YAML frontmatter
    if content.starts_with("---") {
        if let Some(end) = content.find("\n---\n") {
            let frontmatter = &content[4..end];
            prompt = content[end + 5..].to_string();

            // Parse frontmatter
            for line in frontmatter.lines() {
                if line.starts_with("description:") {
                    description = line["description:".len()..].trim().to_string();
                }
                if line.starts_with("tools:") {
                    // Parse tools list
                    let tools_str = line["tools:".len()..].trim();
                    tools = tools_str
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
    }

    (description, prompt, tools)
}

/// Tool to execute Claude Code commands
pub struct ClaudeCommandExecutor {
    config: ClaudeConfig,
}

impl ClaudeCommandExecutor {
    pub fn new(project_dir: impl AsRef<Path>) -> Result<Self> {
        let config = ClaudeConfig::load(project_dir)?;
        Ok(Self { config })
    }

    /// Execute a command by name with given arguments
    pub fn execute(&self, command_name: &str, args: &[String]) -> Result<String> {
        let command = self
            .config
            .get_command(command_name)
            .ok_or_else(|| anyhow!("Command not found: {}", command_name))?;

        // Replace placeholders in prompt with args
        let mut prompt = command.prompt.clone();
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{{{}}}}}", i);
            prompt = prompt.replace(&placeholder, arg);
        }

        Ok(prompt)
    }

    /// Get system prompt enhanced with available skills
    pub fn get_enhanced_system_prompt(&self, base_prompt: &str) -> String {
        if self.config.is_empty() {
            return base_prompt.to_string();
        }

        let mut enhanced = base_prompt.to_string();

        // Add skills context
        if !self.config.skills.is_empty() {
            enhanced.push_str("\n\n## Available Skills\n\n");
            for skill in self.config.skills.values() {
                enhanced.push_str(&format!("### {}\n{}\n\n", skill.name, skill.description));
                enhanced.push_str(&format!("```\n{}\n```\n\n", &skill.content[..skill.content.len().min(500)]));
            }
        }

        // Add available commands
        if !self.config.commands.is_empty() {
            enhanced.push_str("\n## Available Commands\n\n");
            for cmd in self.config.commands.values() {
                enhanced.push_str(&format!("- **{}**: {}\n", cmd.name, cmd.description));
            }
        }

        // Add available subagents
        if !self.config.subagents.is_empty() {
            enhanced.push_str("\n## Available Subagents\n\n");
            for agent in self.config.subagents.values() {
                enhanced.push_str(&format!("- **{}**: {} (tools: {:?})\n",
                    agent.name, agent.description, agent.tools));
            }
        }

        enhanced
    }

    /// Get a subagent prompt by name
    pub fn get_subagent_prompt(&self, name: &str) -> Option<String> {
        self.config.get_subagent(name).map(|a| a.prompt.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_description() {
        let content = "# My Skill\n\nThis is a skill.";
        assert_eq!(extract_description(content), "# My Skill");

        let content2 = "// Another skill\nDescription here.";
        assert_eq!(extract_description(content2), "// Another skill");
    }

    #[test]
    fn test_parse_command_file() {
        let content = "---\ndescription: Test command\n---\n\nThis is the prompt.";
        let (desc, prompt) = parse_command_file(content);
        assert_eq!(desc, "Test command");
        assert_eq!(prompt.trim(), "This is the prompt.");
    }

    #[test]
    fn test_parse_agent_file() {
        let content = "---\ndescription: Test agent\ntools: [read, write, search]\n---\n\nYou are a test agent.";
        let (desc, prompt, tools) = parse_agent_file(content);
        assert_eq!(desc, "Test agent");
        assert_eq!(tools, vec!["read", "write", "search"]);
        assert_eq!(prompt.trim(), "You are a test agent.");
    }
}
