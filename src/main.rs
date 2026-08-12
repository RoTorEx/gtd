use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use clap::Parser;
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
    let mut first = true;
    for (project_name, mut tasks) in grouped {
        if !first {
            println!();
        }
        first = false;
        println!("{}", project_name);

        tasks.sort_by(|a, b| a.added_at.cmp(&b.added_at));

        for task in tasks {
            let added = parse_datetime(&task.added_at)?;
            let age = now.signed_duration_since(added);
            println!(
                "  {:>8}  {:>16}  {}",
                format_age(age),
                added.with_timezone(&Local).format("%Y-%m-%d %H:%M"),
                task.content
            );
        }
    }

    Ok(())
}

async fn fetch_projects(client: &reqwest::Client, token: &str) -> Result<Vec<Project>> {
    let projects: Vec<Project> = client
        .get(format!("{}/projects", TODOIST_API))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .context("failed to parse projects response")?;
    Ok(projects)
}

async fn fetch_tasks(client: &reqwest::Client, token: &str) -> Result<Vec<Task>> {
    let tasks: Vec<Task> = client
        .get(format!("{}/tasks", TODOIST_API))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .context("failed to parse tasks response")?;
    Ok(tasks)
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
