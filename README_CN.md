<p align="right">中文 | <a href="README.md">English</a></p>

<div align="center">
  <h1>arb</h1>
  <p><em>默认极速，按需深入。</em></p>
  <p>一个自带 Shell 套件的 GPU 加速 macOS 终端。<br />零配置。零插件。打开即可编程。</p>
</div>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/rust-2021%2B-000000?style=flat-square&logo=rust" alt="Rust" />
  <a href="https://github.com/szj2ys/arb/stargazers"><img src="https://img.shields.io/github/stars/szj2ys/arb?style=flat-square" alt="GitHub Stars"></a>
  <a href="https://github.com/szj2ys/arb/releases/latest"><img src="https://img.shields.io/github/v/release/szj2ys/arb?style=flat-square" alt="Latest Release"></a>
  <a href="https://github.com/szj2ys/arb/releases"><img src="https://img.shields.io/github/downloads/szj2ys/arb/total?style=flat-square" alt="Downloads"></a>
  <a href="https://github.com/szj2ys/arb/actions"><img src="https://img.shields.io/github/actions/workflow/status/szj2ys/arb/ci.yml?style=flat-square" alt="CI Status"></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/binary-~40%20MB-brightgreen?style=flat-square" alt="Binary Size ~40 MB" />
  <img src="https://img.shields.io/badge/shell%20boot-~100ms-brightgreen?style=flat-square" alt="Shell Boot ~100ms" />
  <img src="https://img.shields.io/badge/config-zero-brightgreen?style=flat-square" alt="Zero Config" />
  <img src="https://img.shields.io/badge/login-not%20required-brightgreen?style=flat-square" alt="No Login Required" />
</p>

<div align="center">
  <br />
  <img src="docs/terminal-preview.svg" alt="arb 终端预览截图" width="800" />
  <br />
  <sub>arb 搭配 Starship 提示符、语法高亮和智能目录导航</sub>
  <br /><br />
</div>

<p align="center">
  <a href="https://szj2ys.github.io/arb/">
    <strong>在线体验 &rarr;</strong>
  </a>
</p>

---

## 特性

- **零配置** -- 精心调校的默认设置，JetBrains Mono 字体、优化的 macOS 字体渲染、流畅动画。打开即可使用。
- **内置 Shell 套件** -- 预装 Starship、z、Delta、语法高亮和自动补全建议。无需额外安装插件。
- **快速轻量** -- ~40 MB 二进制，即时启动，懒加载，精简的 GPU 加速核心。
- **Lua 脚本** -- 保留完整的 Lua 脚本能力，按需进行无限自定义。

---

## 竞品对比

|  | arb | iTerm2 | Alacritty | Kitty | Ghostty | Warp |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **零配置** | 是 | 否 | 否 | 否 | 否 | 部分 |
| **内置 Shell 工具** | 是 | 否 | 否 | 否 | 否 | 部分 |
| **GPU 渲染** | 是 | 部分 | 是 | 是 | 是 | 是 |
| **Lua 脚本** | 是 | 否 | 否 | 是 | 否 | 否 |
| **无需登录** | 是 | 是 | 是 | 是 | 是 | 否 |
| **开源** | 是 | 是 | 是 | 是 | 是 | 否 |
| **二进制大小** | ~40 MB | ~55 MB | ~15 MB | ~30 MB | ~15 MB | ~90 MB |
| **Shell 启动** | ~100ms | ~250ms | ~120ms | ~130ms | ~110ms | ~200ms |

arb 是唯一一个开箱即带完整 Shell 套件（Starship、z、Delta、语法高亮、自动补全建议）的终端 -- 无需插件、无需改 dotfile、无需包管理器折腾。

---

## 为什么选 arb？

### 一条命令，新机器就绑定

大多数终端需要你分别安装提示符、模糊搜索、语法高亮、补全和 diff 工具。arb 把这些全部内置为 Shell 套件。运行 `brew install szj2ys/arb/arb`，打开应用，你的 Shell 就完整了。

### 为 AI 编程工作流而生

AI 辅助开发意味着更多标签页、更多分屏、更多上下文切换。arb 启动极快（Shell 启动 ~100ms），原生支持多标签页和分屏，让你专注于和 AI 工具的对话，而不是等终端响应。

### 只在需要时才配置

arb 提供有主见的默认设置（JetBrains Mono、Arb Dark、流畅动画），适合大多数开发者直接使用。当你确实想修改时，`~/.config/arb/arb.lua` 一个 Lua 配置文件就能完全掌控 -- 没有分散的 dotfile，没有插件管理器，没有依赖链。

### 性能

| 指标 | 典型终端 | arb | 方法 |
| :--- | :--- | :--- | :--- |
| **可执行文件大小** | ~67 MB | ~40 MB | 激进的符号裁剪和功能精简 |
| **资源体积** | ~100 MB | ~80 MB | 资源优化和懒加载 |
| **启动延迟** | 标准 | 即时 | 即时初始化 |
| **Shell 启动** | ~200ms | ~100ms | 优化的环境初始化 |

---

## 快速开始

### 安装

```bash
brew install szj2ys/arb/arb
```

或从 [Releases](https://github.com/szj2ys/arb/releases) 下载最新 `.dmg` 并拖入 Applications。

应用已通过 Apple 公证，无需安全设置即可直接打开。

### 试一试

```bash
# 智能目录跳转（自动学习你的习惯）
z projects    # 瞬间跳转到 ~/projects

# Delta 自动美化 git diff 输出
git diff

# Starship 提示符自动显示 git 状态、版本号、执行时间 -- 开箱即用
```

### 首次启动会发生什么

1. arb 检测你的 Shell（zsh），自动安装内置 Shell 套件 -- Starship 提示符、z 目录跳转、Delta diff 工具、语法高亮和自动补全建议。
2. 在你的 `.zshrc` 中追加一段最小化配置块。arb 只追加，不会覆盖你现有的配置。
3. 一切就绪。打开新标签页，你就拥有了一个完整装备的 Shell。

### 常用命令

```bash
arb doctor   # 检查 Shell 集成是否正常
arb update   # 从命令行检查并应用更新
arb reset    # 移除 arb 管理的 Shell 配置（--yes 非交互式）
```

---

## 内置工具

arb 内置了一套精选的 CLI 工具，预配置好即可投入使用：

| 工具 | 描述 |
| :--- | :--- |
| :rocket: **Starship** | 快速、可自定义的提示符，显示 git 状态、包版本和执行时间 |
| :zap: **z** | 更智能的 cd 命令，学习你最常用的目录以实现即时跳转 |
| :art: **Delta** | 带语法高亮的 git、diff 和 grep 输出分页器 |
| :pencil2: **语法高亮** | 实时命令验证和着色 |
| :bulb: **自动补全建议** | 基于历史记录的智能补全，类似 Fish shell |
| :package: **zsh-completions** | 扩展的命令和子命令补全定义 |

<details>
<summary><strong>快捷键</strong></summary>

arb 提供直觉化的 macOS 原生快捷键：

| 操作 | 快捷键 |
| :--- | :--- |
| 新建标签页 | `Cmd + T` |
| 新建窗口 | `Cmd + N` |
| 垂直分屏 | `Cmd + D` |
| 水平分屏 | `Cmd + Shift + D` |
| 缩放/还原面板 | `Cmd + Shift + Enter` |
| 调整面板大小 | `Cmd + Ctrl + 方向键` |
| 关闭标签页/面板 | `Cmd + W` |
| 切换标签页 | `Cmd + [`, `Cmd + ]` 或 `Cmd + 1-9` |
| 切换面板 | `Cmd + Opt + 方向键` |
| 清屏 | `Cmd + R` |
| 字体大小 | `Cmd + +`, `Cmd + -`, `Cmd + 0` |
| 智能跳转 | `z <目录>` |
| 智能选择 | `z -l <目录>` |
| 最近目录 | `z -t` |

</details>

---

## 配置

### 自定义

arb 通过标准 Lua 脚本完全可配置。

macOS 上，`arb.app/Contents/Resources/arb.lua` 中的内置默认配置仅作为回退，用户配置优先加载。

使用统一的用户配置路径：`~/.config/arb/arb.lua`。

### 更新与重置

- 通过命令行检查/应用更新：`arb update`
- 移除 arb 管理的 Shell 默认配置和集成：`arb reset`（或非交互式 `arb reset --yes`）
- GUI 自动更新检查使用数字版本比较（例如 `0.1.10` 正确地被识别为比 `0.1.9` 更新）。

---

## 支持

- 如果 arb 对你有帮助，欢迎点个 Star 或分享给朋友。
- 有想法或发现 Bug？提个 Issue/PR 或查看 [CONTRIBUTING.md](CONTRIBUTING.md)。

---

## 许可

MIT 许可证，欢迎使用和参与开源。
