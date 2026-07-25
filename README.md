# Tensor Tycoon / 张量大亨

A bilingual terminal strategy game built with Rust and
[`ratatui`](https://ratatui.rs/). One human player competes with one to three
deterministic bots to deploy AI models, collect usage credits, and allocate
Tensor capacity on a compact 24-tile board.

这是一款使用 Rust 和 `ratatui` 开发的双语终端策略游戏。玩家可在 24 格棋盘上
部署 AI 模型、收取使用点数、配置 Tensor，并与 1–3 名固定策略电脑玩家对战。

## Features

- 16 text-generation models from the Qwen, Llama, DeepSeek, and Kimi families
- Prices derived logarithmically from the models' published total parameter counts
- Family bonuses after deploying any three models from the same family
- Even Tensor allocation, release, archiving, restoration, and auctions
- Random Seed events, compute bills, cooldowns, cache hits, and context overflows
- Chinese and English UI, deterministic saves, and a versioned multi-save manager

## Run

```console
cargo run --release
```

The terminal must be at least 98 columns by 28 rows. Release builds target
Windows, Linux, and Apple Silicon macOS.

## Controls

| Key | Action |
| --- | --- |
| `r` | Roll |
| `p` / `a` | Deploy / decline and auction |
| `b` / `a` | Minimum auction bid / pass |
| `m` | Model manager |
| `e` | End turn |
| `s` | Save or open the save command |
| `l` | Switch Chinese/English |
| `:` | Open command palette |
| `?` | Help |
| `q` | Safe quit |

Commands include `roll`, `buy`, `auction`, `bid <amount>`, `end`,
`tensor <tile>`, `untensor <tile>`, `archive <tile>`, `restore <tile>`,
`paycooldown`, `usebypass`, `save [name]`, `load <id>`, `status`, `help`,
and `quit`.

While in cooldown, rolling automatically consumes a bypass token when one is
available. Otherwise, doubles leave cooldown for free; a non-double roll
automatically pays 50 credits before moving. If the fee cannot be raised, the
normal bankruptcy flow applies and the player does not move.

## Development

```console
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

The game engine is independent from the TUI. UI input, text commands, and bot
decisions submit the same validated game actions. Version 1 saves and the former
application data directory are copied and migrated automatically; the old data
is retained as a backup.

## Model names and attribution

Model names and parameter counts refer to third-party model cards hosted on
[Hugging Face](https://huggingface.co/models). Qwen, Llama, DeepSeek, Kimi,
Hugging Face, and related names may be trademarks of their respective owners.
This project is not affiliated with or endorsed by those owners and does not
bundle or redistribute model weights.

## License

GNU Affero General Public License v3.0 only (`AGPL-3.0-only`). See
[LICENSE](LICENSE).
