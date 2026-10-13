# Project Changelog

Tracks real product and release progress.

## [Unreleased]

### Fixed

- Keep completed task markers aligned with active rows so timing and title
  columns do not shift when completing or undoing a task.
- Gray and strike through completed timing labels along with the title and
  description, including selected rows and the Today view.

## [0.2.14] - 2026-10-13

### Added

- Animated request progress for completion, undo, deletion, and refresh, with
  background requests that keep the TUI responsive and prevent duplicate actions.
- Session-long completion feedback: completed rows stay checked and struck
  through in their groups, retaining selection and surviving refresh until exit.
- Undo the latest completion with `u` (including Russian-layout `г`), or reopen
  a selected completed row with `c`. Recurring undo restores the original due
  date and recurrence; failures preserve status and undo history for retry.

## [0.2.13] - 2026-10-13

### Changed

- Use left/right to jump between sections and Shift+left/right for projects,
  so group navigation works with the existing iTerm2 profile without changing
  terminal key bindings or keyboard reporting modes.

## [0.2.12] - 2026-10-13

### Added

- Jump between projects with Cmd+Up/Down, between sections with Option+Up/Down,
  and to the first/last task with Home/End.
- Searchable project/section jump menu on `g`, with full section paths,
  current-view filtering, and Russian keyboard layout support.
- Enable enhanced keyboard reporting on supported terminals for Command/Super
  arrows, restoring the previous mode on exit; document iTerm2 key mappings.

## [0.2.11] - 2026-09-05

### Fixed

- Include overdue tasks in `--today` and show current due dates/times instead
  of creation age, including for recurring tasks, in both TUI and plain output.
- Fetch all task, project, and section pages so the Today view is complete.

## [0.2.10] - 2026-09-05

### Added

- Add `gtd --today` to review tasks due today in the usual TUI or plain listing,
  with project filtering and refresh support.

### Changed

- Routed direct Cargo and IDE build output to
  `~/construction_side/gtd/target`.

## [0.2.9] - 2026-08-22

### Fixed

- Preserve the task-list viewport between frames so upward scrolling moves the
  cursor before the viewport, matching downward scrolling.

## [0.2.8] - 2026-08-22

### Fixed

- Stop task navigation at the first and last items instead of wrapping around.
- Pin project and section context only after their original headers scroll away,
  avoiding duplicate group labels.
- Truncate long project and section names with `...` at the task pane boundary.

## [0.2.7] - 2026-08-22

### Changed

- Keep the project and section of the first visible task pinned above the
  scrolling task list.

## [0.2.6] - 2026-08-22

### Added

- Add five TUI themes—`classic`, `forest`, `sunset`, `ocean`, and `midnight`—
  with distinct palettes and symbol sets, selected through
  `~/.config/gtd/config.toml`.
- Add `gtd themes` to list presets and `gtd theme <name>` to atomically change
  the active theme without requiring Todoist access.

### Changed

- Create a default `theme = "classic"` config during installation while
  preserving an existing config.
- Store every bundled theme in its own `themes/<name>.toml` definition.

## [0.2.5] - 2026-08-22

### Fixed

- Make `Ctrl+C` exit the interactive UI instead of opening the complete-task
  confirmation.

## [0.2.4] - 2026-08-22

### Changed

- Add visual spacing before new project and section groups in the TUI task list.

## [0.2.3] - 2026-08-22

### Fixed

- Open Todoist tasks by their current web URL even though API v1 no longer
  returns a task `url` property.
- Show transient errors and notices in a centered two-second toast instead of
  leaving them indefinitely in the status-bar corner.
- Accept uppercase and Cyrillic-layout variants of TUI shortcut keys.

## [0.2.2] - 2026-08-22

### Added

- Add `gtd update` to download the latest Apple Silicon release, verify its
  SHA-256 checksum and version command, and atomically replace the installed
  executable.

### Changed

- Limit release artifacts to Apple Silicon macOS, matching the machine this
  personal CLI is used on and avoiding unnecessary GitHub Actions runner usage.

## [0.2.1] - 2026-08-22

### Fixed

- Publish GitHub Release assets through a repository-aware action instead of a
  local-context-dependent `gh release` shell command.

## [0.2.0] - 2026-08-22

### Added

- Added an interactive terminal UI with task navigation, a detail pane, refresh,
  browser opening, and confirmations for completing or deleting tasks.
- Added `--plain` for the original grouped non-interactive task listing.
- Added `--version` and `make version` for checking the installed CLI version.
- Added automated GitHub Releases for Linux and macOS on x86_64 and aarch64.

### Changed

- Show task descriptions, due dates, priorities, labels, comment counts, URLs,
  and Todoist ordering metadata where relevant.
- Install the binary under `~/.x-cli-gtd/bin` and keep build output under the
  CLI's runtime directory.
- Made `make check` read-only and removed placeholder release targets until the
  project has a real versioned delivery process.

### Fixed

- Use the rustls-only Reqwest configuration so Linux aarch64 release builds do
  not require a target-specific OpenSSL installation.
