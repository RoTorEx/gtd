# Project Changelog

Tracks real product and release progress.

## [Unreleased]

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
