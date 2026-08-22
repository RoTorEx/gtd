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

## Invariants and boundaries

- The Todoist token is read from the environment, used only for authenticated
  Todoist API requests, and is never persisted or printed.
- Read-only display actions do not mutate Todoist data.
- Completing and deleting tasks are the only Todoist mutations.
- Project matching for `--project` is case-insensitive and exact.
- Missing project or section metadata must not prevent the remaining tasks from
  being displayed.
- Self-update downloads only the named Apple Silicon release archive and its
  checksum from the public `RoTorEx/gtd` GitHub repository, verifies SHA-256,
  verifies the extracted binary through `gtd -V`, and atomically replaces the
  current executable.
- Opening a task uses the API-provided URL when available and otherwise builds
  the current Todoist web URL from the task ID.

## Decision-bearing constants

- Task description previews show at most 30 Unicode characters before an
  ellipsis. This keeps the task list compact while the full description remains
  available in the detail pane. The behavior is protected by unit tests.
- Task ages under 7 days are green, ages from 7 through 29 days are yellow, and
  ages of 30 days or more are red. These thresholds are presentation cues only;
  they do not change Todoist priority or ordering.
- TUI notices and errors appear in the center for 2 seconds. This keeps feedback
  visible without permanently taking space from the task list or status bar.

## Non-goals

- Replacing Todoist as the system of record.
- Creating or editing task content.
- Persisting task data or credentials locally.
- Supporting prebuilt self-update assets on non-Apple-Silicon platforms.

## Code map

- `src/main.rs` — CLI parsing, Todoist API access, grouping, rendering, and task
  actions.
- `src/update.rs` — verified Apple Silicon GitHub Release self-update.
- `Makefile` — stable build, verification, installation, and release commands.
- `.github/workflows/release.yml` — tagged release artifact publication.
