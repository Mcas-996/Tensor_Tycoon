<a id="readme-top"></a>

<div align="center">

# 🎲 Tensor Tycoon

### 一次掷骰、一个模型、一个 Tensor，打造你的 AI 帝国。

一款中英双语终端策略游戏：收购 AI 模型、智胜电脑对手，成为最后的商业巨头。

[English](README.md) · [简体中文](README.zh-CN.md)

[![CI](https://github.com/Mcas-996/Tensor_Tycoon/actions/workflows/ci.yml/badge.svg)](https://github.com/Mcas-996/Tensor_Tycoon/actions/workflows/ci.yml)
[![最新版本](https://img.shields.io/github/v/release/Mcas-996/Tensor_Tycoon?display_name=tag&sort=semver)](https://github.com/Mcas-996/Tensor_Tycoon/releases/latest)
[![许可证：AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-dca282.svg?logo=rust)](https://www.rust-lang.org/)
[![平台](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)](#installation)

[演示](#demo) · [特色](#features) · [安装](#installation) ·
[操作](#controls) · [开发](#development)

</div>

---

<a id="demo"></a>

## 🎬 演示

https://github.com/user-attachments/assets/08a70016-71b2-44c7-bf66-b4c172fbe80d

<a id="features"></a>

## ✨ 游戏特色

| | 特色 | 说明 |
| --- | --- | --- |
| 🌏 | **双语界面** | 随时在英文与简体中文之间切换。 |
| 🤖 | **电脑对手** | 自定义电脑玩家数量，挑战与难度相匹配的策略。 |
| 🧠 | **AI 模型市场** | 收购模型、集齐家族、配置 Tensor，并归档资产。 |
| 🔨 | **实时拍卖** | 被拒绝购买的模型会进入拍卖，所有未破产玩家均可参与。 |
| 🎴 | **策略事件** | 应对事件牌、冷却区、绕过令牌与破产抉择。 |
| 💾 | **持久化存档** | 创建、载入、覆盖并自动迁移带版本的本地存档。 |
| 🖥️ | **跨平台终端界面** | 在 macOS、Linux 或 Windows 的精致 TUI 中游玩。 |

让所有对手破产即可获胜；如果游戏到达回合上限，则净资产最高者获胜。

<a id="installation"></a>

## 🚀 安装

项目为 macOS、Linux 和 Windows 提供预构建安装器。

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Mcas-996/Tensor_Tycoon/releases/latest/download/tensor_tycoon-installer.sh | sh
```

### Windows PowerShell

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/Mcas-996/Tensor_Tycoon/releases/latest/download/tensor_tycoon-installer.ps1 | iex"
```

安装完成后启动游戏：

```console
tensor_tycoon
```

查看命令行选项或当前安装版本：

```console
tensor_tycoon --help
tensor_tycoon --version
```

同时支持简写参数 `-h` 和 `-v`。

### 更新

通过安装器完成的安装会附带更新程序。运行：

```console
tensor_tycoon-update
```

更新程序会检查 GitHub Releases，并在发现新版本时完成安装。

<a id="controls"></a>

## ⌨️ 操作方式

### 键盘快捷键

| 按键 | 使用场景 | 操作 |
| :---: | --- | --- |
| `r` | 移动前 | 掷骰 |
| `p` | 模型购买提示 | 购买模型 |
| `a` | 模型购买提示 | 拒绝购买并开始拍卖 |
| `b` | 轮到你竞拍 | 按最低有效金额出价 |
| `a` | 轮到你竞拍 | 放弃竞拍 |
| `m` | 管理阶段 | 打开模型管理器 |
| `e` | 管理阶段 | 结束回合 |
| `s` | 游戏中 | 保存，或打开保存命令 |
| `l` | 游戏中任意时刻 | 切换中文 / 英文 |
| `:` | 游戏中 | 打开命令面板 |
| `?` | 游戏中 | 打开帮助 |
| `q` | 游戏中 | 安全退出 |

### 命令面板

```text
roll                  buy                   auction
bid <金额>            end                   tensor <格号>
untensor <格号>       archive <格号>        restore <格号>
paycooldown           usebypass             save [名称]
load <id>             status                help
quit
```

## 🧩 规则提示

- **简单模式**为人类玩家提供 2,000 初始点数、三选一掷骰和事件牌，同时让电脑玩家更加保守。
- **模型家族**中拥有任意三个模型后，即可在该家族中均匀配置 Tensor。
- **冷却区**会优先自动消耗绕过令牌；没有令牌时，掷出双数可免费离开，否则会在移动前自动支付 50 点。
- **拍卖**会在玩家拒绝购买模型时开始，所有未破产玩家都可参与。
- **存档**采用带版本格式；旧应用数据目录和版本 1 存档会被自动复制并迁移，原始数据保留为备份。

<a id="development"></a>

## 🛠️ 参与开发

Tensor Tycoon 使用 [Rust](https://www.rust-lang.org/)、
[Ratatui](https://ratatui.rs/) 和
[Crossterm](https://github.com/crossterm-rs/crossterm) 构建。

```console
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

游戏引擎与 TUI 相互独立。键盘输入、文本命令和电脑决策都会提交相同的、经过验证的游戏动作。
更多实现说明请参阅[开发指南](docs/development.md)。

## 🏷️ 模型名称与归属说明

模型名称和参数量来自 [Hugging Face](https://huggingface.co/models)
托管的第三方模型卡。Qwen、Llama、DeepSeek、Kimi、Hugging Face
及相关名称可能是其各自所有者的商标。本项目与这些所有者无关联，也未获得其背书；
本项目不捆绑或再分发任何模型权重。

## 📄 许可证

本项目仅依据 GNU Affero General Public License v3.0
（[`AGPL-3.0-only`](LICENSE)）授权。

<div align="center">

为终端策略玩家与 AI 爱好者打造。

[返回顶部](#readme-top)

</div>
