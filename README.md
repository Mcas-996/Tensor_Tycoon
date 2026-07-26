# Tensor Tycoon 



https://github.com/user-attachments/assets/08a70016-71b2-44c7-bf66-b4c172fbe80d


## Quick Start

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Mcas-996/monopoly_cli/releases/latest/download/tensor_tycoon-installer.sh | sh
```

### Windows PowerShell

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/Mcas-996/monopoly_cli/releases/latest/download/tensor_tycoon-installer.ps1 | iex"
```

After installation, start the game with:

```console
tensor_tycoon
```

To view command-line help or the installed version, run:

```console
tensor_tycoon --help
tensor_tycoon --version
```

The short forms `-h` and `-v` are also supported.

## Updating

Installations created by the shell or PowerShell installer include an updater.
Run:

```console
tensor_tycoon-update
```

The updater checks GitHub Releases and installs a newer version when one is
available.

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

See the [development guide](docs/development.md) for build commands and
implementation notes.

## Model names and attribution

Model names and parameter counts refer to third-party model cards hosted on
[Hugging Face](https://huggingface.co/models). Qwen, Llama, DeepSeek, Kimi,
Hugging Face, and related names may be trademarks of their respective owners.
This project is not affiliated with or endorsed by those owners and does not
bundle or redistribute model weights.

## License

GNU Affero General Public License v3.0 only (`AGPL-3.0-only`). See
[LICENSE](LICENSE).
