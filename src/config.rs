use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub confidence_threshold: f32,
    pub template_directory: PathBuf,
    pub screenshot_directory: PathBuf,
    pub dry_run: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { confidence_threshold: 0.85, template_directory: "assets/templates".into(), screenshot_directory: "screenshots".into(), dry_run: true }
    }
}
impl AppConfig {
    pub fn load_or_create() -> Result<Self, ConfigError> {
        let path = PathBuf::from("config/default.toml");
        if !path.exists() {
            fs::create_dir_all("config")?;
            fs::write(&path, toml::to_string_pretty(&Self::default())?)?;
        }
        let value = toml::from_str(&fs::read_to_string(path)?)?;
        fs::create_dir_all(&value.template_directory)?;
        fs::create_dir_all(&value.screenshot_directory)?;
        Ok(value)
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ConfigError { #[error("filesystem: {0}")] Io(#[from] std::io::Error), #[error("TOML: {0}")] Toml(#[from] toml::de::Error), #[error("TOML: {0}")] Serialize(#[from] toml::ser::Error) }
