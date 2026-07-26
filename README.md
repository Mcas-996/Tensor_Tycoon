<a id="readme-top"></a>

<div align="center">

# 🎲 Tensor Tycoon

### Build an AI empire—one roll, model, and Tensor at a time.

A bilingual terminal strategy game about acquiring AI models, outsmarting bots,
and becoming the last tycoon standing.

[English](README.md) · [简体中文](README.zh-CN.md)

[![CI](https://github.com/Mcas-996/Tensor_Tycoon/actions/workflows/ci.yml/badge.svg)](https://github.com/Mcas-996/Tensor_Tycoon/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/Mcas-996/Tensor_Tycoon?display_name=tag&sort=semver)](https://github.com/Mcas-996/Tensor_Tycoon/releases/latest)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-dca282.svg?logo=rust)](https://www.rust-lang.org/)
[![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)](#installation)

[Demo](#demo) · [Features](#features) · [Installation](#installation) ·
[Controls](#controls) · [Development](#development)

</div>

---

<a id="demo"></a>

## 🎬 Demo

https://github.com/user-attachments/assets/08a70016-71b2-44c7-bf66-b4c172fbe80d

<a id="features"></a>

## ✨ Features

| | Feature | What it means |
| --- | --- | --- |
| 🌏 | **Bilingual interface** | Switch between English and Simplified Chinese at any time. |
| 🤖 | **AI opponents** | Face configurable bots with difficulty-aware strategies. |
| 🧠 | **AI model market** | Acquire models, complete families, allocate Tensors, and archive assets. |
| 🔨 | **Live auctions** | Declined models go to an interactive auction for every solvent player. |
| 💳 | **Credit loans** | Borrow 5,000 credits per loan, repay after ten rounds, and liquidate or auction assets if needed. |
| 🎴 | **Tactical events** | Navigate event cards, cooldowns, bypass tokens, and bankruptcy decisions. |
| 💾 | **Persistent games** | Create, load, overwrite, and migrate versioned local saves. |
| 🖥️ | **Cross-platform TUI** | Play from a polished terminal interface on macOS, Linux, or Windows. |

Win by bankrupting every opponent—or by holding the highest net worth when the
round limit is reached.

<a id="installation"></a>

## 🚀 Installation

Prebuilt installers are published for macOS, Linux, and Windows.

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Mcas-996/Tensor_Tycoon/releases/latest/download/tensor_tycoon-installer.sh | sh
```

### Windows PowerShell

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/Mcas-996/Tensor_Tycoon/releases/latest/download/tensor_tycoon-installer.ps1 | iex"
```

Start the game after installation:

```console
tensor_tycoon
```

Check the available CLI options or installed version:

```console
tensor_tycoon --help
tensor_tycoon --version
```

The short forms `-h` and `-v` are also supported.

### Updating

Installer-based setups include an updater. Run:

```console
tensor_tycoon-update
```

It checks GitHub Releases and installs a newer version when one is available.

<a id="controls"></a>

## ⌨️ Controls

### Keyboard shortcuts

| Key | Context | Action |
| :---: | --- | --- |
| `r` | Before moving | Roll |
| `p` | Purchase offer | Buy the model |
| `a` | Purchase offer | Decline and start an auction |
| `b` | Your auction turn | Place the minimum valid bid |
| `a` | Your auction turn | Pass |
| `m` | Management phase | Open the model manager |
| `n` | Active turn | Take a 5,000-credit loan |
| `e` | Management phase | End the turn |
| `s` | In game | Save, or open the save command |
| `l` | Anywhere in the game | Switch Chinese / English |
| `:` | In game | Open the command palette |
| `?` | In game | Open help |
| `q` | In game | Quit safely |

### Command palette

```text
roll                  buy                   auction
bid <amount>          end                   tensor <tile>
untensor <tile>       archive <tile>        restore <tile>
loan                  sell <tile>           bankrupt
paycooldown           usebypass             save [name]
load <id>             status                help
quit
```

## 🧩 Game notes

- **Easy mode** gives the human 2,000 starting credits, best-of-three rolls and
  cards, while making bots more conservative.
- **Model families** unlock Tensor allocation after you own any three models in
  the same family. Tensors must be distributed evenly.
- **Cooldown** consumes a bypass token automatically when available. Otherwise,
  doubles leave for free and a non-double roll costs 50 credits before movement.
- **Auctions** begin when a purchase is declined and include every player who
  has not gone bankrupt.
- **Loans** grant 5,000 credits each and are due after ten rounds. Loans due
  together are repaid as one amount; outstanding principal does not reduce net
  worth.
- **Loan settlement** allows releasing Tensors, archiving models, or auctioning
  active models with their Tensors. The player may also declare bankruptcy.
- **Saves** use a versioned format; legacy application data and version 1 saves
  are copied and migrated automatically while the originals remain as backups.

<a id="development"></a>

## 🛠️ Development

Tensor Tycoon is built with [Rust](https://www.rust-lang.org/),
[Ratatui](https://ratatui.rs/), and
[Crossterm](https://github.com/crossterm-rs/crossterm).

```console
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

The game engine is independent from the TUI. Keyboard input, text commands, and
bot decisions all submit the same validated game actions. See the
[development guide](docs/development.md) for implementation notes.

## 🏷️ Model names and attribution

Model names and parameter counts refer to third-party model cards hosted on
[Hugging Face](https://huggingface.co/models). Qwen, Llama, DeepSeek, Kimi,
Hugging Face, and related names may be trademarks of their respective owners.
This project is not affiliated with or endorsed by those owners and does not
bundle or redistribute model weights.

## 📄 License

Licensed under the GNU Affero General Public License v3.0 only
([`AGPL-3.0-only`](LICENSE)).

<div align="center">

Made for terminal strategists and AI enthusiasts.

[Back to top](#readme-top)

</div>
