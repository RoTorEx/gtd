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
2. Optionally filter tasks by project name and/or due today or overdue with `--today`.
3. Group and order tasks by project, section, Todoist order, and creation time.
4. Let the user inspect details, open the Todoist URL, refresh the list, complete
   a task, or delete a task.
5. Require an explicit in-application confirmation before completing or deleting
   a task. Keep completed tasks visibly checked and struck through until the
   current TUI exits, so completion feels clear and reassuring. `u` undoes the
   latest completion, repeatedly in reverse order; `c` on a completed row offers
   confirmed reopening of that particular task.
6. On Apple Silicon macOS, let the user replace the running installation with
   the latest published GitHub Release through `gtd update`.
7. Let the user list bundled themes with `gtd themes` and atomically select one
   with `gtd theme <name>` without requiring Todoist credentials or network
   access.

## Invariants and boundaries

- The Todoist token is read from the environment, used only for authenticated
  Todoist API requests, and is never persisted or printed.
- Read-only display actions do not mutate Todoist data.
- Completing, undoing a completion, and deleting tasks are the only Todoist mutations.
- Completed task markers preserve the active row's indentation and column widths.
  Their titles, descriptions, and timing labels (creation age or Today due label)
  are gray and struck through, including while selected. Undo restores normal
  styles without shifting columns.
- Completion feedback is session-only and survives refresh, filters, and other
  mutations. Keep the original completed occurrence for recurring tasks rather
  than showing its next occurrence in this session; the next launch fetches the
  current active occurrence. Undo restores the original recurring due date and
  recurrence through Todoist's Sync uncompletion command (`is_forward = 0`).
  Ordinary tasks use the reopen endpoint. Failed mutations keep the local status
  and undo history intact; refresh failures do not erase a successful mutation.
  Deleting a completed row removes it from session history and cannot be undone.
- After confirmation, Todoist requests run in the background while the TUI
  redraws an animated indicator naming the action. Show progress for completion,
  undo, deletion, and refresh. Block duplicate actions while a request is pending;
  quitting remains available. `N` cancels immediately without a network request.
  Apply completion/undo feedback only after a successful Todoist response.
- Exit key combinations must quit without triggering or confirming a task
  mutation.
- Project matching for `--project` is case-insensitive and exact.
- `--today` means the Todoist Today workload: tasks due today plus all
  overdue tasks. Future, undated, and invalid dates are excluded. Explicit
  timezone offsets are converted to the system timezone; date-only and floating
  times use their stated calendar date. Recurring tasks use their current
  occurrence's due date, never the original creation timestamp.
- In `--today`, the list and details show Today, Yesterday, or the due date,
  including a due time and recurrence marker when present. Past dates are
  marked overdue and use the theme's old/red cue; today's dates use the fresh
  cue. Creation age is hidden in this mode. Plain output uses the same timing
  labels and due dates. The regular view continues to show creation age.
- The today filter combines with `--project`, applies to TUI and plain output,
  and is reevaluated on refresh. Existing project/section grouping and actions
  are preserved.
- Fetch every API page for tasks, projects, and sections using `next_cursor`
  until it is null, so filtering covers the complete active task list.
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
- Task navigation stops at the first and last tasks instead of wrapping around.
- Navigation uses horizontal arrows for groups: Left/Right jumps to the first
  task of the previous/next section across projects; Shift+Left/Right does the
  same for projects (folders). Group navigation
  stops at the boundaries without moving the selection. Home/End selects the
  first/last task.
- Navigation must work without changing the user's terminal behavior, profiles,
  key bindings, or keyboard reporting modes. Cmd/Option arrows are not required,
  because terminals may consume them or encode them incompatibly. Unrecognized
  modifier combinations must not fall back to moving one task.
- The `g` jump menu offers projects and sections with tasks in the current
  filtered view, including `No section`. Case-insensitive substring search
  matches project/section paths so repeated section names remain distinguishable.
  Enter jumps to the group's first task; Escape cancels without moving selection.
  Letters are search input while the menu is open; Ctrl+C always exits.
- The task-list viewport persists between frames, so the selection moves within
  the same scroll padding before the viewport moves in either direction.
- The task list pins only the missing group context for its first visible task:
  no duplicate while both headers remain visible, then the project, then the
  section as each original header scrolls away.
- Project and section headers are truncated with `...` to stay inside the task
  pane at the current terminal width.
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

## Local product usage evidence

Record meaningful workflow outcomes locally, with bounded storage and no source
content. [Usage logging](docs/usage-logging.md) owns the event and retention
contract. Successful API calls, prepared artifacts and user-reported outcomes
must not be presented as broader business success.
