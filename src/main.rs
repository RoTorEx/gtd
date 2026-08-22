use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{ExecutableCommand, cursor};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear as ClearWidget, List, ListItem, Paragraph, Wrap};
use ratatui::{Terminal, backend::CrosstermBackend};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, stdout};
use std::time::Duration;

mod update;

const TODOIST_API: &str = "https://api.todoist.com/api/v1";
const DESC_PREVIEW_LEN: usize = 30;

#[derive(Parser)]
#[command(name = "gtd", version, about = "Review Todoist tasks and their age")]
struct Args {
    /// Show only tasks from this project
    #[arg(short, long)]
    project: Option<String>,

    /// Print the plain grouped list and exit instead of opening the interactive TUI
    #[arg(long)]
    plain: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Install the latest Apple Silicon macOS release
    Update,
}

#[derive(Deserialize, Debug)]
struct PaginatedResponse<T> {
    results: Vec<T>,
}

#[derive(Deserialize, Debug)]
struct Project {
    id: String,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Section {
    id: String,
    name: String,
    #[serde(default)]
    order: i64,
}

#[derive(Deserialize, Debug, Clone)]
struct Task {
    id: String,
    content: String,
    #[serde(default)]
    description: String,
    #[serde(alias = "created_at", default)]
    added_at: String,
    project_id: String,
    section_id: Option<String>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    priority: u8,
    #[serde(default)]
    due: Option<Due>,
    #[serde(default)]
    url: String,
    #[serde(default)]
    comment_count: u64,
    #[serde(default)]
    order: i64,
}

#[derive(Deserialize, Debug, Clone)]
struct Due {
    date: Option<String>,
    datetime: Option<String>,
    string: String,
    #[serde(default)]
    is_recurring: bool,
}

#[derive(Clone, Debug)]
struct TaskItem {
    task: Task,
    project_name: String,
    section_name: String,
    section_order: i64,
}

#[derive(Clone, Debug)]
enum ListEntry {
    ProjectHeader(String),
    SectionHeader(String),
    Task(Box<TaskItem>),
}

#[derive(Clone, Debug)]
enum ConfirmAction {
    Complete(String),
    Delete(String),
}

#[derive(Clone, Debug)]
struct ConfirmState {
    action: ConfirmAction,
    message: String,
}

struct App {
    client: reqwest::Client,
    token: String,
    project_filter: Option<String>,
    project_names: HashMap<String, String>,
    section_names: HashMap<String, (String, i64)>,
    entries: Vec<ListEntry>,
    selectable: Vec<usize>,
    selected: usize,
    max_age: usize,
    confirmation: Option<ConfirmState>,
    message: Option<String>,
}

impl App {
    fn new(
        client: reqwest::Client,
        token: String,
        project_filter: Option<String>,
        projects: Vec<Project>,
        sections: Vec<Section>,
        tasks: Vec<Task>,
    ) -> Self {
        let project_names: HashMap<String, String> =
            projects.into_iter().map(|p| (p.id, p.name)).collect();

        let section_names: HashMap<String, (String, i64)> = sections
            .into_iter()
            .map(|s| (s.id, (s.name, s.order)))
            .collect();

        let mut app = Self {
            client,
            token,
            project_filter,
            project_names,
            section_names,
            entries: Vec::new(),
            selectable: Vec::new(),
            selected: 0,
            max_age: 0,
            confirmation: None,
            message: None,
        };
        app.set_tasks(tasks);
        app
    }

    fn set_tasks(&mut self, tasks: Vec<Task>) {
        let now = Utc::now();
        let mut items: Vec<TaskItem> = tasks
            .into_iter()
            .filter_map(|task| {
                let project_name = self
                    .project_names
                    .get(&task.project_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());

                if let Some(filter) = &self.project_filter
                    && !project_name.eq_ignore_ascii_case(filter)
                {
                    return None;
                }

                let (section_name, section_order) = task
                    .section_id
                    .as_ref()
                    .and_then(|id| self.section_names.get(id).cloned())
                    .unwrap_or_else(|| ("No section".to_string(), 0));

                Some(TaskItem {
                    task,
                    project_name,
                    section_name,
                    section_order,
                })
            })
            .collect();

        items.sort_by(|a, b| {
            a.project_name
                .cmp(&b.project_name)
                .then(a.section_order.cmp(&b.section_order))
                .then(a.section_name.cmp(&b.section_name))
                .then(a.task.order.cmp(&b.task.order))
                .then(a.task.added_at.cmp(&b.task.added_at))
        });

        // Pre-compute max age width across all tasks.
        let mut max_age = 0;
        for item in &items {
            if let Ok(added) = parse_datetime(&item.task.added_at) {
                let age = format_age(now.signed_duration_since(added));
                let age_len = age.chars().count();
                if age_len > max_age {
                    max_age = age_len;
                }
            }
        }
        self.max_age = max_age;

        // Build grouped entries with headers.
        let mut entries: Vec<ListEntry> = Vec::new();
        let mut selectable: Vec<usize> = Vec::new();
        let mut current_project: Option<String> = None;
        let mut current_section: Option<String> = None;

        for item in items {
            if current_project.as_ref() != Some(&item.project_name) {
                current_project = Some(item.project_name.clone());
                current_section = None;
                entries.push(ListEntry::ProjectHeader(item.project_name.clone()));
            }

            if current_section.as_ref() != Some(&item.section_name) {
                current_section = Some(item.section_name.clone());
                if item.section_name != "No section" {
                    entries.push(ListEntry::SectionHeader(item.section_name.clone()));
                }
            }

            selectable.push(entries.len());
            entries.push(ListEntry::Task(Box::new(item)));
        }

        self.entries = entries;
        self.selectable = selectable;
        if self.selected >= self.selectable.len() && !self.selectable.is_empty() {
            self.selected = self.selectable.len() - 1;
        }
    }

    fn selected_task(&self) -> Option<&TaskItem> {
        self.selectable
            .get(self.selected)
            .and_then(|&idx| self.entries.get(idx))
            .and_then(|entry| match entry {
                ListEntry::Task(item) => Some(item.as_ref()),
                _ => None,
            })
    }

    fn move_selection(&mut self, delta: isize) {
        if self.selectable.is_empty() {
            return;
        }
        let len = self.selectable.len();
        let new = if delta < 0 {
            (self.selected + len - delta.unsigned_abs()) % len
        } else {
            (self.selected + delta.unsigned_abs()) % len
        };
        self.selected = new;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if matches!(args.command, Some(Command::Update)) {
        return update::run().await;
    }

    let token = std::env::var("TODOIST_API_TOKEN")
        .context("TODOIST_API_TOKEN environment variable must be set")?;

    let client = reqwest::Client::new();

    let (projects, sections, tasks) = tokio::try_join!(
        fetch_projects(&client, &token),
        fetch_sections(&client, &token),
        fetch_tasks(&client, &token),
    )?;

    if args.plain {
        print_plain_list(&projects, &sections, &tasks, args.project.as_deref())
    } else {
        run_interactive(client, token, projects, sections, tasks, args.project).await
    }
}

async fn run_interactive(
    client: reqwest::Client,
    token: String,
    projects: Vec<Project>,
    sections: Vec<Section>,
    tasks: Vec<Task>,
    project_filter: Option<String>,
) -> Result<()> {
    let mut app = App::new(
        client.clone(),
        token.clone(),
        project_filter,
        projects,
        sections,
        tasks,
    );

    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    let result = tokio::task::block_in_place(|| run_tui(&mut app));

    let _ = stdout.execute(cursor::Show);
    let _ = stdout.execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();

    result
}

struct AltScreenGuard;

impl Drop for AltScreenGuard {
    fn drop(&mut self) {
        let mut stdout = stdout();
        let _ = stdout.execute(cursor::Show);
        let _ = stdout.execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn key_matches(key: KeyCode, latin: char) -> bool {
    let c = match key {
        KeyCode::Char(c) => c,
        _ => return false,
    };

    if c == latin {
        return true;
    }

    // Common Cyrillic equivalents for the same physical keys.
    let cyrillic = match latin {
        'q' => 'й',
        'c' => 'с',
        'd' => 'д',
        'r' => 'р',
        'o' => 'щ',
        'k' => 'л',
        'j' => 'о',
        'y' => 'н',
        'n' => 'т',
        _ => return false,
    };

    c == cyrillic
}

fn run_tui(app: &mut App) -> Result<()> {
    let _guard = AltScreenGuard;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let rt = tokio::runtime::Runtime::new()?;

    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.confirmation.is_some() {
                handle_confirmation_key(app, key.code, &rt)?;
                continue;
            }

            app.message = None;

            match key.code {
                KeyCode::Esc => break,
                key if key_matches(key, 'q') => break,
                key if key_matches(key, 'k') || matches!(key, KeyCode::Up) => {
                    app.move_selection(-1)
                }
                key if key_matches(key, 'j') || matches!(key, KeyCode::Down) => {
                    app.move_selection(1)
                }
                key if key_matches(key, 'c') => {
                    if let Some(item) = app.selected_task() {
                        app.confirmation = Some(ConfirmState {
                            action: ConfirmAction::Complete(item.task.id.clone()),
                            message: format!(
                                "Complete task \"{}\"? [y/N]",
                                truncate_text(&item.task.content, 40)
                            ),
                        });
                    }
                }
                key if key_matches(key, 'd') => {
                    if let Some(item) = app.selected_task() {
                        app.confirmation = Some(ConfirmState {
                            action: ConfirmAction::Delete(item.task.id.clone()),
                            message: format!(
                                "Delete task \"{}\"? [y/N]",
                                truncate_text(&item.task.content, 40)
                            ),
                        });
                    }
                }
                key if key_matches(key, 'r') => {
                    match rt.block_on(fetch_tasks(&app.client, &app.token)) {
                        Ok(tasks) => {
                            app.set_tasks(tasks);
                            app.message = Some("Refreshed tasks".to_string());
                        }
                        Err(e) => app.message = Some(format!("Refresh failed: {e}")),
                    }
                }
                key if key_matches(key, 'o') => {
                    if let Some(item) = app.selected_task() {
                        if item.task.url.is_empty() {
                            app.message = Some("Task has no URL".to_string());
                        } else {
                            match open::that(&item.task.url) {
                                Ok(()) => {
                                    app.message = Some("Opened in browser".to_string());
                                }
                                Err(e) => {
                                    app.message = Some(format!("Failed to open: {e}"));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn handle_confirmation_key(
    app: &mut App,
    code: KeyCode,
    rt: &tokio::runtime::Runtime,
) -> Result<()> {
    let Some(confirm) = app.confirmation.take() else {
        return Ok(());
    };

    if key_matches(code, 'y') {
        match confirm.action {
            ConfirmAction::Complete(id) => {
                match rt.block_on(complete_task(&app.client, &app.token, &id)) {
                    Ok(()) => {
                        app.message = Some(format!("Completed task {id}"));
                        refresh_after_mutation(app, rt)?;
                    }
                    Err(e) => app.message = Some(format!("Complete failed: {e}")),
                }
            }
            ConfirmAction::Delete(id) => {
                match rt.block_on(delete_task(&app.client, &app.token, &id, true)) {
                    Ok(()) => {
                        app.message = Some(format!("Deleted task {id}"));
                        refresh_after_mutation(app, rt)?;
                    }
                    Err(e) => app.message = Some(format!("Delete failed: {e}")),
                }
            }
        }
    } else {
        app.message = Some("Cancelled".to_string());
    }

    Ok(())
}

fn refresh_after_mutation(app: &mut App, rt: &tokio::runtime::Runtime) -> Result<()> {
    match rt.block_on(fetch_tasks(&app.client, &app.token)) {
        Ok(tasks) => {
            let prev_id = app.selected_task().map(|i| i.task.id.clone());
            app.set_tasks(tasks);
            if let Some(id) = prev_id {
                if let Some(new_selected) = app
                    .selectable
                    .iter()
                    .position(|&entry_idx| matches!(&app.entries[entry_idx], ListEntry::Task(item) if item.task.id == id))

                {
                    app.selected = new_selected;
                } else if !app.selectable.is_empty() && app.selected >= app.selectable.len() {
                    app.selected = app.selectable.len() - 1;
                }
            }
        }
        Err(e) => app.message = Some(format!("Refresh failed: {e}")),
    }
    Ok(())
}

fn draw_ui(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main_chunks[0]);

    draw_task_list(frame, app, body_chunks[0]);
    draw_detail_pane(frame, app, body_chunks[1]);
    draw_status_bar(frame, app, main_chunks[1]);

    if let Some(confirm) = &app.confirmation {
        draw_confirmation(frame, confirm, area);
    }
}

fn draw_task_list(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Tasks")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let now = Utc::now();
    let selected_entry_idx = app.selectable.get(app.selected).copied();

    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|entry| {
            let line = match entry {
                ListEntry::ProjectHeader(name) => Line::from(vec![
                    Span::styled("▶ ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        name.clone(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                ListEntry::SectionHeader(name) => Line::from(vec![
                    Span::styled("  ├ ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        name.clone(),
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]),
                ListEntry::Task(item) => {
                    let content = strip_markdown_links(&item.task.content);
                    let desc = truncate_description_preview(&item.task.description);
                    let desc_part = if desc.is_empty() {
                        String::new()
                    } else {
                        format!(" | {desc}")
                    };

                    let age = parse_datetime(&item.task.added_at)
                        .map(|dt| format_age(now.signed_duration_since(dt)))
                        .unwrap_or_else(|_| "?".to_string());
                    let days = parse_datetime(&item.task.added_at)
                        .map(|dt| now.signed_duration_since(dt).num_days())
                        .unwrap_or(0);

                    let age_padded = pad_width(&age, app.max_age);

                    let age_color = match days {
                        ..7 => Color::Green,
                        7..30 => Color::Yellow,
                        _ => Color::Red,
                    };

                    Line::from(vec![
                        Span::styled("    ", Style::default()),
                        Span::styled(format!("{age_padded}  "), Style::default().fg(age_color)),
                        Span::styled(content, Style::default().fg(Color::White)),
                        Span::styled(desc_part, Style::default().fg(Color::Blue)),
                    ])
                }
            };

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_symbol("")
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .scroll_padding(3);

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(selected_entry_idx);
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn pad_width(text: &str, width: usize) -> String {
    let text_width = text.chars().count();
    if text_width >= width {
        text.to_string()
    } else {
        format!("{}{}", text, " ".repeat(width - text_width))
    }
}

fn draw_detail_pane(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Details")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let text = if let Some(item) = app.selected_task() {
        let task = &item.task;
        let added = parse_datetime(&task.added_at)
            .map(|dt| {
                dt.with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_else(|_| task.added_at.clone());
        let age = parse_datetime(&task.added_at)
            .map(|dt| format_age(Utc::now().signed_duration_since(dt)))
            .unwrap_or_else(|_| "?".to_string());

        let mut lines = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Blue)),
                Span::raw(&task.id),
            ]),
            Line::default(),
            Line::from(vec![
                Span::styled(
                    "Title: ",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(strip_markdown_links(&task.content)),
            ]),
            Line::default(),
        ];

        if !task.description.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Description:\n",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )]));
            for paragraph in task.description.lines() {
                lines.push(Line::raw(paragraph.to_string()));
            }
            lines.push(Line::default());
        }

        lines.push(Line::from(vec![
            Span::styled("Project: ", Style::default().fg(Color::Blue)),
            Span::raw(&item.project_name),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Section: ", Style::default().fg(Color::Blue)),
            Span::raw(&item.section_name),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Added: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{added} ({age})")),
        ]));

        if let Some(due) = &task.due {
            let due_str = due
                .datetime
                .clone()
                .or_else(|| due.date.clone())
                .unwrap_or_else(|| due.string.clone());
            let recurring = if due.is_recurring { " (recurring)" } else { "" };
            lines.push(Line::from(vec![
                Span::styled("Due: ", Style::default().fg(Color::Blue)),
                Span::raw(format!("{due_str}{recurring}")),
            ]));
        }

        let priority_label = 5 - task.priority.clamp(1, 4);
        lines.push(Line::from(vec![
            Span::styled("Priority: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("p{priority_label}")),
        ]));

        if !task.labels.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Labels: ", Style::default().fg(Color::Blue)),
                Span::raw(task.labels.join(", ")),
            ]));
        }

        lines.push(Line::from(vec![
            Span::styled("Comments: ", Style::default().fg(Color::Blue)),
            Span::raw(task.comment_count.to_string()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("URL: ", Style::default().fg(Color::Blue)),
            Span::raw(&task.url),
        ]));

        Text::from(lines)
    } else {
        Text::from("No tasks to display.")
    };

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((0, 0));

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let help = "↑/k ↓/j navigate • o open • c complete • d delete • r refresh • q quit";
    let right = app.message.as_deref().unwrap_or("").to_string();

    let msg_width = (right.len() as u16).min(area.width);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(msg_width)])
        .split(area);

    let help_widget = Paragraph::new(help)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Left);

    let msg_widget = Paragraph::new(right)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Right);

    frame.render_widget(help_widget, layout[0]);
    frame.render_widget(msg_widget, layout[1]);
}

fn draw_confirmation(frame: &mut ratatui::Frame, confirm: &ConfirmState, area: Rect) {
    let width = (confirm.message.len() as u16 + 8)
        .min(area.width.saturating_sub(4))
        .max(40);
    let height = 5;
    let popup = centered_rect(width, height, area);

    frame.render_widget(ClearWidget, popup);

    let block = Block::default()
        .title("Confirm")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let text = Text::from(vec![
        Line::raw(""),
        Line::from(vec![Span::raw(&confirm.message)]),
        Line::raw(""),
    ]);

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, popup);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Length((area.height.saturating_sub(height)) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Length((area.width.saturating_sub(width)) / 2),
        ])
        .split(vertical[1])[1]
}

struct TaskRow {
    id: String,
    age: String,
    added: String,
    content: String,
    description: String,
    days: i64,
}

fn print_plain_list(
    projects: &[Project],
    sections: &[Section],
    tasks: &[Task],
    project_filter: Option<&str>,
) -> Result<()> {
    let project_names: HashMap<String, String> = projects
        .iter()
        .map(|p| (p.id.clone(), p.name.clone()))
        .collect();

    let section_names: HashMap<String, (String, i64)> = sections
        .iter()
        .map(|s| (s.id.clone(), (s.name.clone(), s.order)))
        .collect();

    let mut by_project: HashMap<String, Vec<&Task>> = HashMap::new();
    for task in tasks {
        let project_name = project_names
            .get(&task.project_id)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());

        if let Some(filter) = project_filter
            && !project_name.eq_ignore_ascii_case(filter)
        {
            continue;
        }

        by_project
            .entry(task.project_id.clone())
            .or_default()
            .push(task);
    }

    let mut grouped: Vec<(String, Vec<&Task>)> = by_project
        .into_iter()
        .map(|(project_id, tasks)| {
            let name = project_names
                .get(&project_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());
            (name, tasks)
        })
        .collect();
    grouped.sort_by(|a, b| a.0.cmp(&b.0));

    let now = Utc::now();
    for (project_name, tasks) in grouped {
        println!("\n{}", project_header(&project_name));

        let mut by_section: HashMap<Option<String>, Vec<&Task>> = HashMap::new();
        for task in tasks {
            by_section
                .entry(task.section_id.clone())
                .or_default()
                .push(task);
        }

        let mut sections_sorted: Vec<(String, i64, Vec<&Task>)> = by_section
            .into_iter()
            .map(|(section_id, tasks)| {
                let (name, order) = section_id
                    .as_ref()
                    .and_then(|id| section_names.get(id).cloned())
                    .unwrap_or_else(|| ("No section".to_string(), 0));
                (name, order, tasks)
            })
            .collect();
        sections_sorted.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

        let mut first_section = true;
        for (section_name, _, section_tasks) in sections_sorted {
            if !first_section {
                println!();
            }
            first_section = false;

            if section_name != "No section" {
                println!("  {}", section_header(&section_name));
            }

            let mut section_tasks = section_tasks;
            section_tasks.sort_by(|a, b| a.added_at.cmp(&b.added_at));

            let rows: Result<Vec<TaskRow>> = section_tasks
                .iter()
                .map(|task| {
                    let added = parse_datetime(&task.added_at)?;
                    let age = now.signed_duration_since(added);
                    Ok(TaskRow {
                        id: task.id.clone(),
                        age: format_age(age),
                        added: added
                            .with_timezone(&Local)
                            .format("%Y-%m-%d %H:%M")
                            .to_string(),
                        content: strip_markdown_links(&task.content),
                        description: truncate_description_preview(&task.description),
                        days: age.num_days(),
                    })
                })
                .collect();
            let rows = rows?;

            let max_id = rows.iter().map(|r| r.id.len()).max().unwrap_or(0);
            let max_age = rows.iter().map(|r| r.age.len()).max().unwrap_or(0);
            let max_date = rows.iter().map(|r| r.added.len()).max().unwrap_or(0);

            for row in rows {
                let id_padded = format!("{:<width$}", row.id.dimmed(), width = max_id);
                let age_padded = format!("{:<width$}", row.age, width = max_age);
                let date_padded = format!("{:<width$}", row.added, width = max_date);
                let age_colored = match row.days {
                    ..7 => age_padded.green(),
                    7..30 => age_padded.yellow(),
                    _ => age_padded.red(),
                };
                let desc_part = if row.description.is_empty() {
                    String::new()
                } else {
                    format!(" | {}", row.description.dimmed())
                };
                println!(
                    "    {}  {}  {}  {}{}",
                    id_padded,
                    age_colored,
                    date_padded.dimmed(),
                    row.content,
                    desc_part
                );
            }
        }
    }

    Ok(())
}

async fn complete_task(client: &reqwest::Client, token: &str, id: &str) -> Result<()> {
    client
        .post(format!("{}/tasks/{}/close", TODOIST_API, id))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()
        .context("failed to complete task")?;
    Ok(())
}

async fn delete_task(client: &reqwest::Client, token: &str, id: &str, yes: bool) -> Result<()> {
    if !yes {
        print!("Delete task {}? [y/N] ", id.yellow());
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled");
            return Ok(());
        }
    }

    client
        .delete(format!("{}/tasks/{}", TODOIST_API, id))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()
        .context("failed to delete task")?;
    Ok(())
}

fn project_header(name: &str) -> String {
    format!("{} {}", "▶".yellow(), name.bold().cyan())
}

fn section_header(name: &str) -> String {
    format!("{} {}", "├".yellow(), name.italic().bright_white())
}

fn strip_markdown_links(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '[' {
            out.push(c);
            continue;
        }
        let mut label = String::new();
        let mut found_bracket = false;
        for c2 in chars.by_ref() {
            if c2 == ']' {
                found_bracket = true;
                break;
            }
            label.push(c2);
        }
        if !found_bracket {
            out.push('[');
            out.push_str(&label);
            continue;
        }
        if chars.peek() == Some(&'(') {
            chars.next();
            for c2 in chars.by_ref() {
                if c2 == ')' {
                    break;
                }
            }
        }
        out.push_str(&label);
    }
    out
}

fn truncate_description_preview(desc: &str) -> String {
    let desc = desc.trim();
    if desc.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = desc.chars().collect();
    if chars.len() <= DESC_PREVIEW_LEN {
        desc.to_string()
    } else {
        format!(
            "{}...",
            chars[..DESC_PREVIEW_LEN].iter().collect::<String>()
        )
    }
}

fn truncate_text(text: &str, max: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max {
        text.to_string()
    } else {
        format!("{}...", chars[..max].iter().collect::<String>())
    }
}

async fn fetch_sections(client: &reqwest::Client, token: &str) -> Result<Vec<Section>> {
    let response: PaginatedResponse<Section> = client
        .get(format!("{}/sections", TODOIST_API))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .context("failed to parse sections response")?;
    Ok(response.results)
}

async fn fetch_projects(client: &reqwest::Client, token: &str) -> Result<Vec<Project>> {
    let response: PaginatedResponse<Project> = client
        .get(format!("{}/projects", TODOIST_API))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .context("failed to parse projects response")?;
    Ok(response.results)
}

async fn fetch_tasks(client: &reqwest::Client, token: &str) -> Result<Vec<Task>> {
    let response: PaginatedResponse<Task> = client
        .get(format!("{}/tasks", TODOIST_API))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .context("failed to parse tasks response")?;
    Ok(response.results)
}

fn parse_datetime(raw: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .with_context(|| format!("failed to parse datetime: {}", raw))
}

fn format_age(dur: chrono::TimeDelta) -> String {
    let days = dur.num_days();
    let hours = dur.num_hours() - days * 24;
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else {
        format!("{}h", hours)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    #[test]
    fn strips_markdown_link_targets() {
        assert_eq!(
            strip_markdown_links("Read [the guide](https://example.com) today"),
            "Read the guide today"
        );
    }

    #[test]
    fn preserves_unclosed_markdown_label() {
        assert_eq!(strip_markdown_links("Read [the guide"), "Read [the guide");
    }

    #[test]
    fn truncates_description_on_character_boundaries() {
        let input = "абвгдеёжзийклмнопрстуфхцчшщъыьэюя";
        let preview = truncate_description_preview(input);

        assert_eq!(preview.chars().count(), DESC_PREVIEW_LEN + 3);
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn leaves_short_description_unchanged() {
        assert_eq!(truncate_description_preview("  short note  "), "short note");
    }

    #[test]
    fn formats_age_in_days_and_hours() {
        assert_eq!(format_age(TimeDelta::hours(51)), "2d 3h");
        assert_eq!(format_age(TimeDelta::hours(5)), "5h");
    }

    #[test]
    fn parses_rfc3339_datetime_as_utc() {
        let parsed = parse_datetime("2026-08-22T12:30:00+03:00").unwrap();

        assert_eq!(parsed.to_rfc3339(), "2026-08-22T09:30:00+00:00");
    }
}
