use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use colored::Colorize;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use serde::Deserialize;
use std::collections::HashMap;

const TODOIST_API: &str = "https://api.todoist.com/api/v1";

#[derive(Parser)]
#[command(name = "gtd", about = "Review Todoist tasks and their age")]
struct Args {
    /// Show only tasks from this project
    #[arg(short, long)]
    project: Option<String>,
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
struct Task {
    content: String,
    #[serde(alias = "created_at")]
    added_at: String,
    project_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let token = std::env::var("TODOIST_API_TOKEN")
        .context("TODOIST_API_TOKEN environment variable must be set")?;

    let client = reqwest::Client::new();

    let (projects, tasks) = tokio::try_join!(
        fetch_projects(&client, &token),
        fetch_tasks(&client, &token),
    )?;

    let project_names: HashMap<String, String> =
        projects.into_iter().map(|p| (p.id, p.name)).collect();

    let mut by_project: HashMap<String, Vec<Task>> = HashMap::new();
    for task in tasks {
        let project_name = project_names
            .get(&task.project_id)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());

        if let Some(filter) = &args.project
            && !project_name.eq_ignore_ascii_case(filter)
        {
            continue;
        }

        by_project
            .entry(task.project_id.clone())
            .or_default()
            .push(task);
    }

    let mut grouped: Vec<(String, Vec<Task>)> = by_project
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
    for (project_name, mut tasks) in grouped {
        println!("\n{}", project_header(&project_name));

        tasks.sort_by(|a, b| a.added_at.cmp(&b.added_at));

        let mut table = Table::new();
        table
            .set_header(vec!["Age", "Added", "Task"])
            .set_content_arrangement(ContentArrangement::DynamicFullWidth);

        for task in tasks {
            let added = parse_datetime(&task.added_at)?;
            let age = now.signed_duration_since(added);
            let age_str = format_age(age);
            let age_color = age_color(age.num_days());
            let date_str = added
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M")
                .to_string();
            let content = strip_markdown_links(&task.content);

            table.add_row(vec![
                Cell::new(age_str).fg(age_color),
                Cell::new(date_str).fg(Color::DarkGrey),
                Cell::new(content),
            ]);
        }

        println!("{table}");
    }

    Ok(())
}

fn project_header(name: &str) -> String {
    format!("{} {}", "▶".yellow(), name.bold().cyan())
}

fn age_color(days: i64) -> Color {
    match days {
        ..7 => Color::Green,
        7..30 => Color::Yellow,
        _ => Color::Red,
    }
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
        // Skip the URL in parentheses, if present.
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
