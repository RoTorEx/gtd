use anyhow::{Context, Result, bail};
use ratatui::style::Color;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const PRESETS: [(&str, &str); 5] = [
    ("classic", include_str!("../themes/classic.toml")),
    ("forest", include_str!("../themes/forest.toml")),
    ("sunset", include_str!("../themes/sunset.toml")),
    ("ocean", include_str!("../themes/ocean.toml")),
    ("midnight", include_str!("../themes/midnight.toml")),
];

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    pub tasks_title: String,
    pub details_title: String,
    pub project_icon: String,
    pub section_icon: String,
    pub task_icon: String,
    pub help_separator: String,
    pub notice_title: String,
    pub error_title: String,
    pub confirm_title: String,
    pub list_border: Color,
    pub detail_border: Color,
    pub label: Color,
    pub project_icon_color: Color,
    pub project: Color,
    pub section_icon_color: Color,
    pub section: Color,
    pub task: Color,
    pub description: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub status: Color,
    pub age_fresh: Color,
    pub age_aging: Color,
    pub age_old: Color,
    pub info: Color,
    pub error: Color,
    pub confirm: Color,
}

#[derive(Debug, Deserialize)]
struct ThemeFile {
    name: String,
    tasks_title: String,
    details_title: String,
    project_icon: String,
    section_icon: String,
    task_icon: String,
    help_separator: String,
    notice_title: String,
    error_title: String,
    confirm_title: String,
    list_border: String,
    detail_border: String,
    label: String,
    project_icon_color: String,
    project: String,
    section_icon_color: String,
    section: String,
    task: String,
    description: String,
    selected_bg: String,
    selected_fg: String,
    status: String,
    age_fresh: String,
    age_aging: String,
    age_old: String,
    info: String,
    error: String,
    confirm: String,
}

impl ThemeFile {
    fn into_theme(self) -> Result<Theme> {
        Ok(Theme {
            name: self.name,
            tasks_title: self.tasks_title,
            details_title: self.details_title,
            project_icon: self.project_icon,
            section_icon: self.section_icon,
            task_icon: self.task_icon,
            help_separator: self.help_separator,
            notice_title: self.notice_title,
            error_title: self.error_title,
            confirm_title: self.confirm_title,
            list_border: parse_color(&self.list_border)?,
            detail_border: parse_color(&self.detail_border)?,
            label: parse_color(&self.label)?,
            project_icon_color: parse_color(&self.project_icon_color)?,
            project: parse_color(&self.project)?,
            section_icon_color: parse_color(&self.section_icon_color)?,
            section: parse_color(&self.section)?,
            task: parse_color(&self.task)?,
            description: parse_color(&self.description)?,
            selected_bg: parse_color(&self.selected_bg)?,
            selected_fg: parse_color(&self.selected_fg)?,
            status: parse_color(&self.status)?,
            age_fresh: parse_color(&self.age_fresh)?,
            age_aging: parse_color(&self.age_aging)?,
            age_old: parse_color(&self.age_old)?,
            info: parse_color(&self.info)?,
            error: parse_color(&self.error)?,
            confirm: parse_color(&self.confirm)?,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Config {
    theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "classic".to_string(),
        }
    }
}

pub fn load() -> Result<Theme> {
    let name = active_name()?;
    get(&name)
}

pub fn get(name: &str) -> Result<Theme> {
    find(name)?.ok_or_else(|| unknown_theme_error(name))
}

pub fn print_available() -> Result<()> {
    let active = get(&active_name()?)?.name;
    println!("Available themes:");
    for theme in presets()? {
        let marker = if theme.name == active { "*" } else { " " };
        println!(
            "{marker} {:<10} {} {} {}",
            theme.name,
            theme.project_icon.trim(),
            theme.section_icon.trim(),
            theme.task_icon.trim()
        );
    }
    println!("\n* active");
    Ok(())
}

pub fn set_active(name: &str) -> Result<PathBuf> {
    get(name)?;

    let path = config_path()
        .context("cannot locate the config file; set GTD_CONFIG or HOME before changing themes")?;
    let existing = if path.exists() {
        fs::read_to_string(&path)
            .with_context(|| format!("failed to read config {}", path.display()))?
    } else {
        include_str!("../config.example.toml").to_string()
    };
    let updated = update_config(&existing, name)?;
    write_config(&path, &updated)?;
    Ok(path)
}

fn active_name() -> Result<String> {
    let Some(path) = config_path() else {
        return Ok(Config::default().theme);
    };
    if !path.exists() {
        return Ok(Config::default().theme);
    }

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("failed to parse config {}", path.display()))?;
    Ok(config.theme)
}

fn presets() -> Result<Vec<Theme>> {
    PRESETS
        .iter()
        .map(|(expected_name, contents)| {
            let definition: ThemeFile = toml::from_str(contents)
                .with_context(|| format!("failed to parse bundled theme {expected_name}"))?;
            if definition.name != *expected_name {
                bail!(
                    "bundled theme {expected_name} declares the name {:?}",
                    definition.name
                );
            }
            definition.into_theme()
        })
        .collect()
}

fn find(name: &str) -> Result<Option<Theme>> {
    Ok(presets()?.into_iter().find(|theme| theme.name == name))
}

fn update_config(contents: &str, name: &str) -> Result<String> {
    let mut document = contents
        .parse::<toml_edit::DocumentMut>()
        .context("failed to parse existing config")?;
    document["theme"] = toml_edit::value(name);
    Ok(document.to_string())
}

fn write_config(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    let temporary = parent.join(format!(".gtd-config-{}.tmp", std::process::id()));

    let result = (|| -> Result<()> {
        fs::write(&temporary, contents)
            .with_context(|| format!("failed to write temporary config {}", temporary.display()))?;
        fs::rename(&temporary, path)
            .with_context(|| format!("failed to replace config {}", path.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn config_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("GTD_CONFIG").filter(|path| !path.is_empty()) {
        return Some(PathBuf::from(path));
    }
    if let Some(root) = std::env::var_os("XDG_CONFIG_HOME").filter(|path| !path.is_empty()) {
        return Some(PathBuf::from(root).join("gtd/config.toml"));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/gtd/config.toml"))
}

fn parse_color(name: &str) -> Result<Color> {
    let color = match name {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "dark_gray" => Color::DarkGray,
        "light_red" => Color::LightRed,
        "light_green" => Color::LightGreen,
        "light_yellow" => Color::LightYellow,
        "light_blue" => Color::LightBlue,
        "light_magenta" => Color::LightMagenta,
        "light_cyan" => Color::LightCyan,
        "white" => Color::White,
        _ => bail!("unknown theme color {name:?}"),
    };
    Ok(color)
}

fn unknown_theme_error(name: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "unknown theme {name:?}; choose one of: {}",
        PRESETS
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_file_parses_and_matches_its_filename() {
        let themes = presets().unwrap();

        assert_eq!(themes.len(), PRESETS.len());
        for ((expected, _), theme) in PRESETS.iter().zip(themes) {
            assert_eq!(theme.name, *expected);
        }
    }

    #[test]
    fn config_defaults_to_current_classic_theme() {
        let config: Config = toml::from_str("").unwrap();

        assert_eq!(config.theme, "classic");
    }

    #[test]
    fn changing_theme_preserves_other_config_and_comments() {
        let updated = update_config(
            "# Keep me\nproject = \"Work\"\ntheme = \"classic\"\n",
            "ocean",
        )
        .unwrap();

        assert!(updated.contains("# Keep me"));
        assert!(updated.contains("project = \"Work\""));
        assert!(updated.contains("theme = \"ocean\""));
    }

    #[test]
    fn rejects_unknown_theme_names() {
        let error = get("missing").unwrap_err().to_string();

        assert!(error.contains("unknown theme \"missing\""));
        assert!(error.contains("classic, forest, sunset, ocean, midnight"));
    }

    #[test]
    fn every_theme_has_a_distinct_palette_and_symbol_set() {
        let themes = presets().unwrap();

        for (index, theme) in themes.iter().enumerate() {
            for other in &themes[index + 1..] {
                assert_ne!(
                    (
                        theme.list_border,
                        theme.detail_border,
                        theme.project,
                        theme.section,
                        theme.selected_bg,
                    ),
                    (
                        other.list_border,
                        other.detail_border,
                        other.project,
                        other.section,
                        other.selected_bg,
                    ),
                    "{} and {} share a palette",
                    theme.name,
                    other.name
                );
                assert_ne!(theme.tasks_title, other.tasks_title);
                assert_ne!(theme.details_title, other.details_title);
                assert_ne!(theme.project_icon, other.project_icon);
                assert_ne!(theme.section_icon, other.section_icon);
                assert_ne!(theme.task_icon, other.task_icon);
                assert_ne!(theme.help_separator, other.help_separator);
                assert_ne!(theme.notice_title, other.notice_title);
                assert_ne!(theme.error_title, other.error_title);
                assert_ne!(theme.confirm_title, other.confirm_title);
            }
        }
    }
}
