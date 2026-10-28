# gtd

An interactive Rust CLI that lists your active Todoist tasks, shows their age,
and lets you complete or delete them from the terminal.

## Installation

On an Apple Silicon Mac, download `gtd-macos-aarch64.tar.gz` from
[GitHub Releases](https://github.com/RoTorEx/gtd/releases), extract `gtd`, and
put it somewhere on your `PATH`. Other platforms can build from source.

To build from source, install a current stable Rust toolchain, clone this
repository, and run `make install`. The binary is installed at
`~/.toolbelt/gtd/bin/gtd`, and the installer adds that directory to your shell
profile's `PATH`.

The installer also creates `~/.toolbelt/gtd/config.toml` when it does not exist
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

# 5. (Optional) Show tasks due today and overdue
gtd --today
gtd --today --project "Work"

# 6. (Optional) Print the plain grouped list instead of opening the TUI
gtd --plain
```

Check the running binary with `gtd --version` or the source checkout with
`make version`. Once installed, update it from the latest Apple Silicon GitHub
Release with:

```bash
gtd update
```

`--today` shows today's workload, including overdue tasks, as in Todoist.
It displays the current due date/time (Today, Yesterday, or a date), marks
overdue dates and recurring tasks, and hides creation age in this mode.
Future tasks and tasks without a due date are excluded. Dates use the local
timezone. It works with `--plain` and stays active when refreshing the TUI.

## Interactive controls

| Key | Action |
|-----|--------|
| `↑` / `↓` or `k` / `j` | Move selection |
| `Shift+←` / `Shift+→` | Jump to previous / next project (folder) |
| `←` / `→` | Jump to previous / next section, across projects |
| `g` (or `п` on the Russian keyboard layout) | Search projects and sections and jump directly |
| `Home` / `End` | Jump to first / last task |
| `o` | Open selected task in browser |
| `c` | Complete selected task, or reopen a completed row (with confirmation) |
| `u` (or `г` on the Russian keyboard layout) | Undo the latest completion in this session |
| `d` | Delete selected task (with confirmation) |
| `r` | Refresh the task list |
| `q` / `Esc` / `Ctrl+C` | Quit |

After confirming with `Y`, an animated indicator shows the current action while
waiting for Todoist. It also appears during undo, deletion, and refresh. Further
actions are blocked until the request finishes; quitting remains available.
`N` cancels immediately without sending a request.

Completed tasks stay in their group with a checkmark and struck-through text
until you quit GTD, including after refresh. The selection stays on the completed
row. Press `u` to undo the latest completion; repeat to undo earlier completions,
or select a completed row and press `c` to reopen that specific task. Undo becomes
available only after Todoist confirms completion; failed undo remains retryable.
For recurring tasks, the completed occurrence stays visible during this session,
and undo restores its original due date and repeat rule. After restarting GTD,
the list shows Todoist's current active tasks. Deletion cannot be undone.

The left pane shows each task's title plus up to 30 characters of its
description. The right pane shows the full description, project, section, age,
due date, priority, labels, comments, URL, and ID.
Navigation stops at the first and last tasks. As group headers scroll away, the
missing project and then section are pinned at the top without duplicating
headers that remain visible. Long group names are truncated with `...` to stay
inside the left pane. Upward and downward navigation use the same cursor-first
scroll behavior.

Group jumps select the first task in the adjacent group and stop at the list
boundaries. Tasks without a section form a `No section` group. Only groups with
tasks in the current filtered view are offered. Press `g` to open the jump menu,
type part of a project or section name (case-insensitive), choose with `↑` / `↓`,
and press `Enter`. Section paths include the project name to distinguish repeated
names. `Esc` cancels without moving the task selection; `Ctrl+C` still quits.

Group navigation uses standard arrow sequences, including iTerm2's default
Shift+arrow mappings. It requires no terminal configuration changes and does
not change keyboard reporting modes or terminal key bindings.

## Themes

List the bundled themes and change the active one without opening the TUI:

```bash
gtd themes
gtd theme ocean
```

The active theme is stored in `~/.toolbelt/gtd/config.toml`:

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
- `CHANGELOG.md` — release progress.
- `config.example.toml` — default user configuration and available theme names.
- `themes/*.toml` — one bundled palette and symbol definition per theme.
- `.vibe/kernel/*.md` — committed local workflow instructions.
- `.github/workflows/release.yml` — tagged GitHub Release publication.

## Commands

See `Makefile`.

## License

[MIT](LICENSE)

## Local usage logs

See [usage logging](docs/usage-logging.md) for product events, privacy, paths,
rotation and interpretation limits.
