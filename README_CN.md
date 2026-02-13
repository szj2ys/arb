<p align="right">中文 | <a href="README.md">English</a></p>

<div align="center">
  <h1>Kaku</h1>
  <p><em>一个为 AI 编程打造的快速、开箱即用的终端。</em></p>
</div>

<p align="center">
  <a href="https://github.com/tw93/Kaku/stargazers"><img src="https://img.shields.io/github/stars/tw93/Kaku?style=flat-square" alt="Stars"></a>
  <a href="https://github.com/tw93/Kaku/releases"><img src="https://img.shields.io/github/v/tag/tw93/Kaku?label=version&style=flat-square" alt="Version"></a>
  <a href="LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://github.com/tw93/Kaku/commits"><img src="https://img.shields.io/github/commit-activity/m/tw93/Kaku?style=flat-square" alt="Commits"></a>
  <a href="https://twitter.com/HiTw93"><img src="https://img.shields.io/badge/follow-Tw93-red?style=flat-square&logo=Twitter" alt="Twitter"></a>
</p>

<p align="center">
  <img src="assets/kaku.jpeg" alt="Kaku 截图" width="1000" />
  <br/>
  Kaku 是 <a href="https://github.com/wez/wezterm">WezTerm</a> 的深度定制分支，专注于开箱即用的体验。
</p>

## 特性

- **零配置**: 精心调校的默认设置，JetBrains Mono 字体、优化的 macOS 字体渲染、流畅动画。
- **内置 Shell 套件**: 预装 Starship、z、Delta、语法高亮和自动补全建议。
- **快速轻量**: 二进制体积缩减 40%，即时启动，懒加载，精简的 GPU 加速核心。
- **Lua 脚本**: 保留 WezTerm 完整的 Lua 引擎，支持无限自定义。

## 快速开始

1. [下载 Kaku DMG](https://github.com/tw93/Kaku/releases/latest) 并拖入 Applications
2. 或使用 Homebrew 安装：`brew install tw93/tap/kakuterm`
3. 打开 Kaku。应用已通过 Apple 公证，无需安全设置即可直接打开
4. 首次启动时，Kaku 会自动配置你的 Shell 环境

## 使用指南

Kaku 提供直觉化的 macOS 原生快捷键：

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

## 配置

Kaku 内置了一套精选的 CLI 工具，预配置好即可投入使用：

- **Starship**: 快速、可自定义的提示符，显示 git 状态、包版本和执行时间。
- **z**: 更智能的 cd 命令，学习你最常用的目录以实现即时跳转。
- **Delta**: 带语法高亮的 git、diff 和 grep 输出分页器。
- **语法高亮**: 实时命令验证和着色。
- **自动补全建议**: 基于历史记录的智能补全，类似 Fish shell。

### 自定义

Kaku 通过标准 Lua 脚本完全可配置，100% 兼容 WezTerm 配置。配置文件按以下优先级加载：

1. **内置配置**: `Kaku.app/Contents/Resources/kaku.lua` 处理默认设置。
2. **用户覆盖**: 创建 `~/.config/kaku/kaku.lua` 并返回你的配置表。

## 为什么做 Kaku？

我重度依赖命令行进行工作和个人项目。我做的工具如 [Mole](https://github.com/tw93/mole) 和 [Pake](https://github.com/tw93/pake) 都体现了这一点。

我用了多年 Alacritty，但它不支持多标签页，在 AI 辅助编程场景下越来越不方便。Kitty 在审美和布局上有些怪癖。Ghostty 有潜力但字体渲染还需改进。Warp 臃肿且需要登录。iTerm2 可靠但略显老态，深度定制也不够方便。

WezTerm 强大且可扩展，我非常感谢它的引擎。但我想要一个开箱即用、无需大量配置的环境——而且要快得多、轻得多。

所以我做了 Kaku：快速、精致、即刻可用。

### 性能

| 指标 | 上游 | Kaku | 方法 |
| :--- | :--- | :--- | :--- |
| **可执行文件大小** | ~67 MB | ~40 MB | 激进的符号裁剪和功能精简 |
| **资源体积** | ~100 MB | ~80 MB | 资源优化和懒加载 |
| **启动延迟** | 标准 | 即时 | 即时初始化 |
| **Shell 启动** | ~200ms | ~100ms | 优化的环境初始化 |

通过激进地裁剪未使用的功能、懒加载配色方案和 Shell 优化来实现。

## 支持

- 如果 Kaku 对你有帮助，给仓库点个 Star 或 [分享给朋友](https://twitter.com/intent/tweet?url=https://github.com/tw93/Kaku&text=Kaku%20-%20A%20fast%20terminal%20built%20for%20AI%20coding.)。
- 有想法或发现 Bug？提个 Issue/PR 或查看 [CONTRIBUTING.md](CONTRIBUTING.md)。
- 喜欢 Kaku？<a href="https://miaoyan.app/cats.html?name=Kaku" target="_blank">请 Tw93 喝杯可乐</a> 支持项目！

## 许可

MIT 许可证，欢迎使用和参与开源。
