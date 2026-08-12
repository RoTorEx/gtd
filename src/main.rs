use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use colored::Colorize;
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
struct Section {
    id: String,
    name: String,
    #[serde(default)]
    order: i64,
}

#[derive(Deserialize, Debug)]
struct Task {
    content: String,
    #[serde(alias = "created_at")]
    added_at: String,
    project_id: String,
    section_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let token = std::env::var("TODOIST_API_TOKEN")
        .context("TODOIST_API_TOKEN environment variable must be set")?;

    let client = reqwest::Client::new();

    let (projects, sections, tasks) = tokio::try_join!(
        fetch_projects(&client, &token),
        fetch_sections(&client, &token),
        fetch_tasks(&client, &token),
    )?;

    let project_names: HashMap<String, String> =
        projects.into_iter().map(|p| (p.id, p.name)).collect();

    let section_names: HashMap<String, (String, i64)> = sections
        .into_iter()
        .map(|s| (s.id, (s.name, s.order)))
        .collect();

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
    for (project_name, tasks) in grouped {
        println!("\n{}", project_header(&project_name));

        let mut by_section: HashMap<Option<String>, Vec<Task>> = HashMap::new();
        for task in tasks {
            by_section
                .entry(task.section_id.clone())
                .or_default()
                .push(task);
        }

        let mut sections_sorted: Vec<(String, i64, Vec<Task>)> = by_section
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
        for (section_name, _, mut section_tasks) in sections_sorted {
            if !first_section {
                println!();
            }
            first_section = false;

            if section_name != "No section" {
                println!("  {}", section_header(&section_name));
            }

            section_tasks.sort_by(|a, b| a.added_at.cmp(&b.added_at));

            let rows: Result<Vec<(String, String, String, i64)>> = section_tasks
                .iter()
                .map(|task| {
                    let added = parse_datetime(&task.added_at)?;
                    let age = now.signed_duration_since(added);
                    Ok((
                        format_age(age),
                        added
                            .with_timezone(&Local)
                            .format("%Y-%m-%d %H:%M")
                            .to_string(),
                        strip_markdown_links(&task.content),
                        age.num_days(),
                    ))
                })
                .collect();
            let rows = rows?;

            let max_age = rows.iter().map(|r| r.0.len()).max().unwrap_or(0);
            let max_date = rows.iter().map(|r| r.1.len()).max().unwrap_or(0);

            for (age_str, date_str, content, days) in rows {
                let age_padded = format!("{:<width$}", age_str, width = max_age);
                let date_padded = format!("{:<width$}", date_str, width = max_date);
                let age_colored = match days {
                    ..7 => age_padded.green(),
                    7..30 => age_padded.yellow(),
                    _ => age_padded.red(),
                };
                println!("    {}  {}  {}", age_colored, date_padded.dimmed(), content);
            }
        }
    }

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
