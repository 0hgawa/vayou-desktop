use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSettings {
    pub volume: f64,
    pub speed: f64,
    pub remember_position: bool,
    pub auto_play: bool,
    #[serde(default = "default_lang")]
    pub language: String,
    #[serde(default)]
    pub recent_files: Vec<RecentFile>,
    #[serde(default)]
    pub subtitle_style: SubtitleStyleSettings,
    #[serde(default = "default_translate_lang")]
    pub translate_lang: String,
    #[serde(default)]
    pub keybindings: std::collections::HashMap<String, String>,
}

fn default_translate_lang() -> String { "pt".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleStyleSettings {
    pub font: String,
    pub size: u32,
    pub color: String,
    pub border_color: String,
    pub border_size: u32,
    pub position: u32,
    #[serde(default)]
    pub bold: bool,
}

fn default_lang() -> String { "en".into() }

impl Default for SubtitleStyleSettings {
    fn default() -> Self {
        Self {
            font: "Segoe UI".into(),
            size: 55,
            color: "#ffffff".into(),
            border_color: "#000000".into(),
            border_size: 3,
            position: 100,
            bold: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: String,
    pub title: String,
    pub position: f64,
    pub timestamp: i64,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            volume: 100.0,
            speed: 1.0,
            remember_position: true,
            auto_play: true,
            language: "en".into(),
            recent_files: Vec::new(),
            subtitle_style: SubtitleStyleSettings::default(),
            translate_lang: default_translate_lang(),
            keybindings: std::collections::HashMap::new(),
        }
    }
}

impl PlayerSettings {
    fn config_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("Vayou")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), AppError> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Config(e.to_string()))?;
        std::fs::write(&path, json)?;
        info!("Settings saved to {}", path.display());
        Ok(())
    }

    pub fn touch_recent(&mut self, path: &str, title: &str, position: f64) {
        self.recent_files.retain(|f| f.path != path);
        self.recent_files.insert(0, RecentFile {
            path: path.to_string(),
            title: title.to_string(),
            position,
            timestamp: chrono::Utc::now().timestamp(),
        });
        self.recent_files.truncate(20);
    }

    pub fn get_saved_position(&self, path: &str) -> Option<f64> {
        self.recent_files
            .iter()
            .find(|f| f.path == path)
            .map(|f| f.position)
            .filter(|&p| p > 1.0)
    }
}
