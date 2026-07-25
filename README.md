# Terminal Tycoon / 终端大富翁

A bilingual, original property-trading game for the terminal, built with Rust and
[`ratatui`](https://ratatui.rs/). One human player competes against one to three
deterministic bots on a compact 20-tile board.

这是一款使用 Rust 和 `ratatui` 开发的原创双语终端地产游戏。单人玩家可以在
20 格棋盘上与 1–3 名固定策略电脑玩家对战。

## Features

- Full-screen perimeter board with Chinese and English UI
- Buying, rent, color groups, evenly built houses, selling, mortgages and auctions
- Doubles, jail, two stations, two utilities and a 12-card original event deck
- Last-player-standing victory or configurable 20–500 round limit
- Versioned multi-save manager in the platform-standard user data directory
- Hotkeys plus an in-game `:` command palette
- Serializable deterministic random state, so loading resumes the exact game

No names, board layout, artwork or card text from commercial Monopoly editions are
included.

## Run

```console
cargo run --release
```

The terminal must be at least 80 columns by 24 rows. The generated release
configuration targets Windows, Linux and Apple Silicon macOS.

## Controls

| Key | Action |
| --- | --- |
| `r` | Roll |
| `p` / `a` | Purchase / decline and auction |
| `b` / `a` | Minimum auction bid / pass |
| `m` | Asset manager |
| `e` | End turn |
| `s` | Save or open the save command |
| `l` | Switch Chinese/English |
| `:` | Open command palette |
| `?` | Help |
| `q` | Safe quit |

Commands include `roll`, `buy`, `auction`, `bid <amount>`, `end`,
`build <tile>`, `sell <tile>`, `mortgage <tile>`, `unmortgage <tile>`,
`payjail`, `usecard`, `save [name]`, `load <id>`, `status`, `help`, and
`quit`.

## Development

```console
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

The game engine is independent from the TUI. UI input, text commands and bot
decisions all submit the same validated game actions, while save files contain the
entire engine state.

## License

GNU Affero General Public License v3.0 only (`AGPL-3.0-only`). See [LICENSE](LICENSE).
