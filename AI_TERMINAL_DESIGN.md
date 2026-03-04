# Arb AI Terminal Architecture Design

## Overview
Transform arb into an AI-powered terminal with Warp-like AI assistance and Claude Code-like Team Agent support.

## Core Features

### 1. AI Command Assistant (类似 Warp AI)
- **Natural Language to Command**: User types in plain English, AI suggests shell commands
- **Command Explanation**: AI explains what a command does before execution
- **Error Diagnosis**: AI analyzes error output and suggests fixes
- **Smart Completion**: AI-powered command and flag completion

### 2. Agent Mode (自主代理)
- **Task Execution**: Agent can execute multi-step tasks autonomously
- **File Operations**: Read, write, search files
- **Command Execution**: Run commands and analyze output
- **Context Awareness**: Maintains session context and history

### 3. Team Agent (类似 Claude Code)
- **Multi-Agent Orchestration**: Run multiple agents in parallel
- **Specialized Roles**: Code reviewer, architect, tester, etc.
- **Message Passing**: Agents communicate via message bus
- **Shared State**: Common context and file system view

### 4. Custom LLM Configuration
- **Custom API URL**: Support any OpenAI-compatible endpoint
- **Custom API Key**: User provides their own key
- **Model Selection**: Configurable model per agent
- **Local LLM Support**: Support for Ollama, LM Studio, etc.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Arb AI Terminal                         │
├─────────────────────────────────────────────────────────────┤
│  UI Layer (Rust/GPU)                                        │
│  ├── Terminal Renderer                                      │
│  ├── AI Overlay Panel                                       │
│  ├── Agent Status Widget                                    │
│  └── Chat Interface                                         │
├─────────────────────────────────────────────────────────────┤
│  AI Core (Rust + Tokio)                                     │
│  ├── LLM Client (OpenAI-compatible)                         │
│  ├── Context Manager                                        │
│  ├── Command Generator                                      │
│  └── Session Manager                                        │
├─────────────────────────────────────────────────────────────┤
│  Agent System (Rust)                                        │
│  ├── Agent Runtime                                          │
│  ├── Task Scheduler                                         │
│  ├── Message Bus                                            │
│  └── State Store                                            │
├─────────────────────────────────────────────────────────────┤
│  Tools (Rust)                                               │
│  ├── File Tools (read/write/search)                         │
│  ├── Shell Tools (execute/capture)                          │
│  ├── Git Tools (status/diff/commit)                         │
│  └── LSP Tools (code intel)                                 │
└─────────────────────────────────────────────────────────────┘
```

## Configuration

### User Config (~/.config/arb/arb.lua)
```lua
-- AI Configuration
config.ai = {
  -- Default LLM provider
  provider = {
    name = "dashscope",  -- or "openai", "anthropic", "ollama"
    api_url = "https://coding.dashscope.aliyuncs.com/v1",
    api_key = os.getenv("DASHSCOPE_API_KEY"),
    model = "kimi-k2.5",
  },

  -- Agent-specific LLM overrides
  agents = {
    code_reviewer = {
      model = "kimi-k2.5",
      temperature = 0.1,
    },
    architect = {
      model = "kimi-k2.5",
      temperature = 0.7,
    },
  },

  -- Features
  features = {
    command_suggestions = true,
    auto_complete = true,
    error_diagnosis = true,
    agent_mode = true,
  },
}

-- Team Agent Configuration
config.team = {
  max_parallel_agents = 4,
  message_timeout_ms = 30000,
  shared_context = true,
}
```

## Commands

### AI Commands
```bash
arb ai "deploy this to production"           # Natural language command
arb ai explain "kubectl get pods"            # Explain command
arb ai fix                                   # Fix last error
arb ai chat                                  # Start AI chat session
```

### Agent Commands
```bash
arb agent "refactor auth module"             # Run autonomous agent
arb agent status                             # Check agent status
arb agent stop                               # Stop current agent
```

### Team Agent Commands
```bash
arb team start                               # Start team session
arb team add reviewer                        # Add code reviewer agent
arb team add tester                          # Add tester agent
arb team status                              # Show team status
arb team broadcast "task done"               # Broadcast to all agents
arb team stop                                # Stop all agents
```

### Configuration Commands
```bash
arb ai config set-url https://...            # Set LLM URL
arb ai config set-key sk-...                 # Set API key
arb ai config set-model kimi-k2.5            # Set model
arb ai config test                           # Test connection
```

## Implementation Phases

### Phase 0: Foundation (Week 1)
1. LLM Client abstraction
2. Configuration system
3. Basic AI command interface

### Phase 1: AI Assistant (Week 2)
1. Natural language to command
2. Command explanation
3. Error diagnosis
4. AI chat panel

### Phase 2: Agent Mode (Week 3)
1. Agent runtime
2. Tool system (file, shell, git)
3. Autonomous task execution
4. Context management

### Phase 3: Team Agent (Week 4)
1. Multi-agent orchestration
2. Message bus
3. Shared state
4. Agent specialization

## Technical Stack

- **Language**: Rust
- **Async Runtime**: Tokio
- **HTTP Client**: reqwest
- **LLM Protocol**: OpenAI-compatible API
- **Serialization**: serde + serde_json
- **Configuration**: Lua (compatible with existing arb config)
- **State Storage**: SQLite (local) or custom backend

## Security Considerations

1. API keys stored in OS keychain (Keychain on macOS)
2. Command execution requires user confirmation by default
3. File access restricted to project directory
4. Network access configurable (allowlist/blocklist)
5. Audit log of all AI actions

## Testing Configuration

```yaml
# Test LLM Configuration
provider:
  name: dashscope
  api_url: https://coding.dashscope.aliyuncs.com/v1
  api_key: sk-sp-3a9cf8cb9a714f67bec0f464a13bcb35
  model: kimi-k2.5
```
