use anyhow::{Context, Result, bail};
use ratatui::style::Color;
use serde::Deserialize;
use std::path::PathBuf;

pub const THEME_NAMES: [&str; 5] = ["classic", "forest", "sunset", "ocean", "midnight"];

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub name: &'static str,
    pub tasks_title: &'static str,
    pub details_title: &'static str,
    pub project_icon: &'static str,
    pub section_icon: &'static str,
    pub task_icon: &'static str,
    pub help_separator: &'static str,
    pub notice_title: &'static str,
    pub error_title: &'static str,
    pub confirm_title: &'static str,
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
            tasks_title: "▣ Tasks",
            details_title: "▤ Details",
            project_icon: "▶ ",
            section_icon: "  ├ ",
            task_icon: "  · ",
            help_separator: "•",
            notice_title: "✓ Notice",
            error_title: "! Error",
            confirm_title: "? Confirm",
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
            tasks_title: "♣ Tasks",
            details_title: "⌁ Details",
            project_icon: "◆ ",
            section_icon: "  └ ",
            task_icon: "  ∙ ",
            help_separator: "◇",
            notice_title: "❧ Notice",
            error_title: "⚠ Error",
            confirm_title: "⌘ Confirm",
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
            tasks_title: "☀ Tasks",
            details_title: "✺ Details",
            project_icon: "◉ ",
            section_icon: "  ╰ ",
            task_icon: "  ◦ ",
            help_separator: "✦",
            notice_title: "✹ Notice",
            error_title: "‼ Error",
            confirm_title: "✷ Confirm",
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

    fn ocean() -> Self {
        Self {
            name: "ocean",
            tasks_title: "≈ Tasks",
            details_title: "≋ Details",
            project_icon: "◈ ",
            section_icon: "  ╭ ",
            task_icon: "  ○ ",
            help_separator: "~",
            notice_title: "≃ Notice",
            error_title: "⚓ Error",
            confirm_title: "◌ Confirm",
            list_border: Color::Cyan,
            detail_border: Color::Blue,
            label: Color::LightCyan,
            project_icon_color: Color::LightBlue,
            project: Color::LightCyan,
            section_icon_color: Color::Cyan,
            section: Color::LightBlue,
            task: Color::White,
            description: Color::Cyan,
            selected_bg: Color::Cyan,
            selected_fg: Color::Black,
            status: Color::LightCyan,
            age_fresh: Color::LightCyan,
            age_aging: Color::LightBlue,
            age_old: Color::Magenta,
            info: Color::LightCyan,
            error: Color::LightRed,
            confirm: Color::LightBlue,
        }
    }

    fn midnight() -> Self {
        Self {
            name: "midnight",
            tasks_title: "☾ Tasks",
            details_title: "✧ Details",
            project_icon: "★ ",
            section_icon: "  ┆ ",
            task_icon: "  ⋅ ",
            help_separator: "⋆",
            notice_title: "☄ Notice",
            error_title: "× Error",
            confirm_title: "☽ Confirm",
            list_border: Color::DarkGray,
            detail_border: Color::LightBlue,
            label: Color::LightMagenta,
            project_icon_color: Color::LightBlue,
            project: Color::LightBlue,
            section_icon_color: Color::DarkGray,
            section: Color::LightMagenta,
            task: Color::Gray,
            description: Color::DarkGray,
            selected_bg: Color::DarkGray,
            selected_fg: Color::White,
            status: Color::Gray,
            age_fresh: Color::LightBlue,
            age_aging: Color::LightMagenta,
            age_old: Color::LightRed,
            info: Color::LightBlue,
            error: Color::LightRed,
            confirm: Color::LightMagenta,
        }
    }

    fn named(name: &str) -> Option<Self> {
        match name {
            "classic" => Some(Self::classic()),
            "forest" => Some(Self::forest()),
            "sunset" => Some(Self::sunset()),
            "ocean" => Some(Self::ocean()),
            "midnight" => Some(Self::midnight()),
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
    fn every_theme_has_a_distinct_palette_and_symbol_set() {
        let themes: Vec<_> = THEME_NAMES
            .iter()
            .map(|name| Theme::named(name).unwrap())
            .collect();

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
