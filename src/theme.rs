use anyhow::{Context, Result, bail};
use ratatui::style::Color;
use serde::Deserialize;
use std::path::PathBuf;

pub const THEME_NAMES: [&str; 3] = ["classic", "forest", "sunset"];

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub name: &'static str,
    pub tasks_title: &'static str,
    pub details_title: &'static str,
    pub project_icon: &'static str,
    pub section_icon: &'static str,
    pub task_icon: &'static str,
    pub help_separator: &'static str,
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

impl Theme {
    pub fn classic() -> Self {
        Self {
            name: "classic",
            tasks_title: "Tasks",
            details_title: "Details",
            project_icon: "▶ ",
            section_icon: "  ├ ",
            task_icon: "    ",
            help_separator: "•",
            list_border: Color::Blue,
            detail_border: Color::Magenta,
            label: Color::Blue,
            project_icon_color: Color::Yellow,
            project: Color::Cyan,
            section_icon_color: Color::Yellow,
            section: Color::Magenta,
            task: Color::White,
            description: Color::Blue,
            selected_bg: Color::Blue,
            selected_fg: Color::White,
            status: Color::Gray,
            age_fresh: Color::Green,
            age_aging: Color::Yellow,
            age_old: Color::Red,
            info: Color::Green,
            error: Color::Red,
            confirm: Color::Red,
        }
    }

    fn forest() -> Self {
        Self {
            name: "forest",
            tasks_title: "◆ Tasks",
            details_title: "◆ Details",
            project_icon: "◆ ",
            section_icon: "  └ ",
            task_icon: "  • ",
            help_separator: "◇",
            list_border: Color::Green,
            detail_border: Color::LightGreen,
            label: Color::LightGreen,
            project_icon_color: Color::Yellow,
            project: Color::LightGreen,
            section_icon_color: Color::Green,
            section: Color::Yellow,
            task: Color::White,
            description: Color::DarkGray,
            selected_bg: Color::Green,
            selected_fg: Color::Black,
            status: Color::LightGreen,
            age_fresh: Color::LightGreen,
            age_aging: Color::Yellow,
            age_old: Color::LightRed,
            info: Color::LightGreen,
            error: Color::LightRed,
            confirm: Color::Yellow,
        }
    }

    fn sunset() -> Self {
        Self {
            name: "sunset",
            tasks_title: "◉ Tasks",
            details_title: "◉ Details",
            project_icon: "◉ ",
            section_icon: "  ╰ ",
            task_icon: "  ◦ ",
            help_separator: "·",
            list_border: Color::LightRed,
            detail_border: Color::LightMagenta,
            label: Color::Yellow,
            project_icon_color: Color::Yellow,
            project: Color::LightRed,
            section_icon_color: Color::LightRed,
            section: Color::LightMagenta,
            task: Color::White,
            description: Color::LightRed,
            selected_bg: Color::LightMagenta,
            selected_fg: Color::Black,
            status: Color::Yellow,
            age_fresh: Color::Cyan,
            age_aging: Color::Yellow,
            age_old: Color::LightRed,
            info: Color::Yellow,
            error: Color::LightRed,
            confirm: Color::LightRed,
        }
    }

    fn named(name: &str) -> Option<Self> {
        match name {
            "classic" => Some(Self::classic()),
            "forest" => Some(Self::forest()),
            "sunset" => Some(Self::sunset()),
            _ => None,
        }
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
            theme: Theme::classic().name.to_string(),
        }
    }
}

pub fn load() -> Result<Theme> {
    let Some(path) = config_path() else {
        return Ok(Theme::classic());
    };
    if !path.exists() {
        return Ok(Theme::classic());
    }

    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("failed to parse config {}", path.display()))?;

    let Some(theme) = Theme::named(&config.theme) else {
        bail!(
            "unknown theme {:?} in {}; choose one of: {}",
            config.theme,
            path.display(),
            THEME_NAMES.join(", ")
        );
    };
    Ok(theme)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_all_documented_themes() {
        for name in THEME_NAMES {
            assert_eq!(Theme::named(name).unwrap().name, name);
        }
    }

    #[test]
    fn config_defaults_to_current_classic_theme() {
        let config: Config = toml::from_str("").unwrap();

        assert_eq!(config.theme, "classic");
    }

    #[test]
    fn themes_have_distinct_colors_and_icons() {
        let classic = Theme::classic();
        let forest = Theme::named("forest").unwrap();
        let sunset = Theme::named("sunset").unwrap();

        assert_ne!(classic.list_border, forest.list_border);
        assert_ne!(forest.list_border, sunset.list_border);
        assert_ne!(classic.project_icon, forest.project_icon);
        assert_ne!(forest.project_icon, sunset.project_icon);
    }
}
