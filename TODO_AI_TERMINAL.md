# Arb AI Terminal Implementation

## Phase 0: Foundation (本周完成)

### Tasks

#### 1. LLM Client 抽象层
- [ ] 创建 `arb/src/ai/` 模块目录
- [ ] 实现 `LLMProvider` trait
- [ ] 实现 OpenAI-compatible 客户端
- [ ] 支持自定义 API URL 和 Key
- [ ] 支持流式响应
- [ ] 添加单元测试

#### 2. 配置系统扩展
- [ ] 扩展 `config` crate 支持 AI 配置
- [ ] 添加 `ai.lua` 配置 schema
- [ ] 支持多 provider 配置
- [ ] 添加配置验证

#### 3. 基础 AI 命令接口
- [ ] 创建 `arb ai` 子命令
- [ ] 实现基础 chat 功能
- [ ] 添加 `arb ai config` 命令
- [ ] 测试 LLM 连接

### 测试配置
```yaml
provider: dashscope
api_url: https://coding.dashscope.aliyuncs.com/v1
api_key: sk-sp-3a9cf8cb9a714f67bec0f464a13bcb35
model: kimi-k2.5
```

## Phase 1: AI Assistant (下周)
- 自然语言转命令
- 命令解释
- 错误诊断
- AI 聊天面板

## Phase 2: Agent Mode (第三周)
- Agent 运行时
- 工具系统
- 自主任务执行

## Phase 3: Team Agent (第四周)
- 多 Agent 编排
- 消息总线
- 共享状态

---

### Worktrees
- `T1-ai-core` - Phase 0: Foundation
- `T1-ai-assistant` - Phase 1: AI Assistant
- `T1-ai-agent` - Phase 2: Agent Mode
- `T1-ai-team` - Phase 3: Team Agent
