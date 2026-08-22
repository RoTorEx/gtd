# gtd

An interactive Rust CLI that lists your active Todoist tasks, shows their age,
and lets you complete or delete them from the terminal.

## Installation

On an Apple Silicon Mac, download `gtd-macos-aarch64.tar.gz` from
[GitHub Releases](https://github.com/RoTorEx/gtd/releases), extract `gtd`, and
put it somewhere on your `PATH`. Other platforms can build from source.

To build from source, install a current stable Rust toolchain, clone this
repository, and run `make install`. The binary is installed at
`~/.x-cli-gtd/bin/gtd`, and the installer adds that directory to your shell
profile's `PATH`.

The installer also creates `~/.config/gtd/config.toml` when it does not exist
(or `$XDG_CONFIG_HOME/gtd/config.toml` when that variable is set). Existing
configuration is preserved on later installs.

## Quickstart

```bash
# 1. Set your Todoist API token
export TODOIST_API_TOKEN="your-token-here"

# 2. Build and install locally
make install

# 3. Review your tasks in the interactive TUI
gtd

# 4. (Optional) Show only one project
gtd --project "Work"

# 5. (Optional) Print the plain grouped list instead of opening the TUI
gtd --plain
```

Check the running binary with `gtd --version` or the source checkout with
`make version`. Once installed, update it from the latest Apple Silicon GitHub
Release with:

```bash
gtd update
```

## Interactive controls

| Key | Action |
|-----|--------|
| `↑` / `↓` or `k` / `j` | Move selection |
| `o` | Open selected task in browser |
| `c` | Complete selected task (with confirmation) |
| `d` | Delete selected task (with confirmation) |
| `r` | Refresh the task list |
| `q` / `Esc` / `Ctrl+C` | Quit |

The left pane shows each task's title plus up to 30 characters of its
description. The right pane shows the full description, project, section, age,
due date, priority, labels, comments, URL, and ID.
Navigation stops at the first and last tasks. As group headers scroll away, the
missing project and then section are pinned at the top without duplicating
headers that remain visible. Long group names are truncated with `...` to stay
inside the left pane. Upward and downward navigation use the same cursor-first
scroll behavior.

## Themes

List the bundled themes and change the active one without opening the TUI:

```bash
gtd themes
gtd theme ocean
```

The active theme is stored in `~/.config/gtd/config.toml`:

```toml
theme = "classic"
```

| Theme | Palette and interface symbols |
|-------|-------------------------------|
| `classic` | Blue, cyan, and magenta with `▣`/`▤` panels and `▶`, `├`, `·` list markers |
| `forest` | Green and yellow with `♣`/`⌁` panels and `◆`, `└`, `∙` list markers |
| `sunset` | Red, magenta, and yellow with `☀`/`✺` panels and `◉`, `╰`, `◦` list markers |
| `ocean` | Cyan and blue with `≈`/`≋` panels and `◈`, `╭`, `○` list markers |
| `midnight` | Dark gray, blue, and magenta with `☾`/`✧` panels and `★`, `┆`, `⋅` list markers |

Each theme also has its own status separator and notice, error, and confirmation
symbols; symbol sets are not shared between themes.

`gtd theme` preserves other config entries and rejects unknown names. To keep
the config elsewhere, set `GTD_CONFIG` to its file path. When
`XDG_CONFIG_HOME` is set, the default path is
`$XDG_CONFIG_HOME/gtd/config.toml`.

## Development

```bash
make check
make version
```

`make check` verifies formatting, lint, tests, and compilation without rewriting
source files. Use `make fmt` explicitly to format.

The repository keeps a local copy of its agent workflow kernel. Maintainers can
refresh it with:

```bash
make vibe-kernel-set
make vibe-pull
```

`.vibe/KERNEL_SOURCE` contains a machine-local path and is gitignored.

## Docs map

- `AGENTS.md` — agent router.
- `BUSINESS.md` — product purpose, flows, and boundaries.
- `TASK.md` — task queue (agents process and remove completed tasks).
- `CHANGELOG.md` — release progress.
- `config.example.toml` — default user configuration and available theme names.
- `themes/*.toml` — one bundled palette and symbol definition per theme.
- `.vibe/kernel/*.md` — committed local workflow instructions.
- `.github/workflows/release.yml` — tagged GitHub Release publication.

## Commands

See `Makefile`.

## License

[MIT](LICENSE)
