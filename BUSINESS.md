# Product Truth

## Purpose

`gtd` is a personal terminal client for reviewing active Todoist tasks. It makes
task age and context visible and lets the user act on one task without switching
to the Todoist application.

## Actor

- The user owns the Todoist account and supplies a Todoist API token through the
  `TODOIST_API_TOKEN` environment variable.

## Core concepts

- A task belongs to a Todoist project and may belong to a section.
- Task age is the elapsed time since Todoist's creation timestamp.
- The interactive view is the default experience; `--plain` provides a
  non-interactive grouped listing.
- A theme controls the TUI palette and its project, section, task, pane, and
  status symbols without changing task behavior.

## Main flows

1. Fetch active projects, sections, and tasks from Todoist over HTTPS.
2. Optionally filter tasks by project name.
3. Group and order tasks by project, section, Todoist order, and creation time.
4. Let the user inspect details, open the Todoist URL, refresh the list, complete
   a task, or delete a task.
5. Require an explicit in-application confirmation before completing or deleting
   a task.
6. On Apple Silicon macOS, let the user replace the running installation with
   the latest published GitHub Release through `gtd update`.
7. Let the user list bundled themes with `gtd themes` and atomically select one
   with `gtd theme <name>` without requiring Todoist credentials or network
   access.

## Invariants and boundaries

- The Todoist token is read from the environment, used only for authenticated
  Todoist API requests, and is never persisted or printed.
- Read-only display actions do not mutate Todoist data.
- Completing and deleting tasks are the only Todoist mutations.
- Exit key combinations must quit without triggering or confirming a task
  mutation.
- Project matching for `--project` is case-insensitive and exact.
- Missing project or section metadata must not prevent the remaining tasks from
  being displayed.
- Self-update downloads only the named Apple Silicon release archive and its
  checksum from the public `RoTorEx/gtd` GitHub repository, verifies SHA-256,
  verifies the extracted binary through `gtd -V`, and atomically replaces the
  current executable.
- Opening a task uses the API-provided URL when available and otherwise builds
  the current Todoist web URL from the task ID.
- Installation creates a default config only when none exists; it must not
  overwrite the user's selected theme.
- Theme selection accepts only bundled names and preserves unrelated config
  entries and comments.

## Decision-bearing constants

- Task description previews show at most 30 Unicode characters before an
  ellipsis. This keeps the task list compact while the full description remains
  available in the detail pane. The behavior is protected by unit tests.
- Task ages under 7 days are green, ages from 7 through 29 days are yellow, and
  ages of 30 days or more are red. These thresholds are presentation cues only;
  they do not change Todoist priority or ordering.
- TUI notices and errors appear in the center for 2 seconds. This keeps feedback
  visible without permanently taking space from the task list or status bar.
- The task list leaves a blank row before a project or section that follows
  tasks, so group boundaries remain visually distinct.
- The task list keeps the project and section of its first visible task pinned
  above the scrollable rows, so tasks never lose their group context when their
  original headers scroll away.
- The supported theme names are `classic`, `forest`, `sunset`, `ocean`, and
  `midnight`. `classic` is the default for compatibility. Every theme must have
  its own palette and symbols for panels, projects, sections, tasks, status,
  notices, errors, and confirmations; pairwise distinction is protected by unit
  tests.
- Each bundled theme is defined in its own `themes/<name>.toml` file and embedded
  into the binary, so listing and switching themes does not depend on runtime
  asset files.

## Non-goals

- Replacing Todoist as the system of record.
- Creating or editing task content.
- Persisting task data or credentials locally.
- Supporting prebuilt self-update assets on non-Apple-Silicon platforms.

## Code map

- `src/main.rs` — CLI parsing, Todoist API access, grouping, rendering, and task
  actions.
- `src/update.rs` — verified Apple Silicon GitHub Release self-update.
- `src/theme.rs` and `themes/*.toml` — config selection plus bundled palette and
  symbol definitions.
- `Makefile` — stable build, verification, installation, and release commands.
- `.github/workflows/release.yml` — tagged release artifact publication.
