<p align="right">中文 | <a href="README.md">English</a></p>

<div align="center">
  <h1>arb</h1>
  <p><em>一个为 AI 编程打造的快速、开箱即用的终端。</em></p>
</div>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/rust-2021%2B-000000?style=flat-square&logo=rust" alt="Rust" />
  <a href="https://github.com/szj2ys/arb/stargazers"><img src="https://img.shields.io/github/stars/szj2ys/arb?style=flat-square" alt="GitHub Stars"></a>
  <a href="https://github.com/szj2ys/arb/releases/latest"><img src="https://img.shields.io/github/v/release/szj2ys/arb?style=flat-square" alt="Latest Release"></a>
  <a href="https://github.com/szj2ys/arb/releases"><img src="https://img.shields.io/github/downloads/szj2ys/arb/total?style=flat-square" alt="Downloads"></a>
  <a href="https://github.com/szj2ys/arb/actions"><img src="https://img.shields.io/github/actions/workflow/status/szj2ys/arb/ci.yml?style=flat-square" alt="CI Status"></a>
</p>

## 特性

- **零配置**: 精心调校的默认设置，JetBrains Mono 字体、优化的 macOS 字体渲染、流畅动画。
- **内置 Shell 套件**: 预装 Starship、z、Delta、语法高亮和自动补全建议。
- **快速轻量**: 二进制体积缩减 40%，即时启动，懒加载，精简的 GPU 加速核心。
- **Lua 脚本**: 保留完整的 Lua 脚本能力，支持无限自定义。

## 为什么做 arb？

我重度依赖命令行进行工作和个人项目。

我用了多年 Alacritty，但它不支持多标签页，在 AI 辅助编程场景下越来越不方便。Kitty 在审美和布局上有些怪癖。Ghostty 有潜力但字体渲染还需改进。Warp 臃肿且需要登录。iTerm2 可靠但略显老态，深度定制也不够方便。

我想要一个开箱即用、无需大量配置的环境——而且要快得多、轻得多。

所以我做了 arb：快速、精致、即刻可用。

### 性能

| 指标 | 上游 | arb | 方法 |
| :--- | :--- | :--- | :--- |
| **可执行文件大小** | ~67 MB | ~40 MB | 激进的符号裁剪和功能精简 |
| **资源体积** | ~100 MB | ~80 MB | 资源优化和懒加载 |
| **启动延迟** | 标准 | 即时 | 即时初始化 |
| **Shell 启动** | ~200ms | ~100ms | 优化的环境初始化 |

通过激进地裁剪未使用的功能、懒加载配色方案和 Shell 优化来实现。

## 快速开始

1. 下载最新 DMG 并拖入 Applications。
2. 或使用 Homebrew 安装：
   ```bash
   brew install szj2ys/arb/arb
   ```
3. 打开 arb。应用已通过 Apple 公证，无需安全设置即可直接打开
4. 首次启动时，arb 会自动配置你的 Shell 环境

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

## 配置

### 自定义

arb 通过标准 Lua 脚本完全可配置。

macOS 上，`arb.app/Contents/Resources/arb.lua` 中的内置默认配置仅作为回退，用户配置优先加载。

使用统一的用户配置路径：`~/.config/arb/arb.lua`。

### 更新与重置

- 通过命令行检查/应用更新：`arb update`
- 移除 arb 管理的 Shell 默认配置和集成：`arb reset`（或非交互式 `arb reset --yes`）
- GUI 自动更新检查使用数字版本比较（例如 `0.1.10` 正确地被识别为比 `0.1.9` 更新）。

## 支持

- 如果 arb 对你有帮助，欢迎点个 Star 或分享给朋友。
- 有想法或发现 Bug？提个 Issue/PR 或查看 [CONTRIBUTING.md](CONTRIBUTING.md)。


## 许可

MIT 许可证，欢迎使用和参与开源。
