# Development

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
