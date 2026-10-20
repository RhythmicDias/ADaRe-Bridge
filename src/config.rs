use std::fs;
use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_device_name")]
    pub device_name: String,

    #[serde(default = "default_target_process")]
    pub target_process: String,

    #[serde(default = "default_jog_cc")]
    pub jog_cc: u8,

    #[serde(default = "default_zoom_cc")]
    pub zoom_cc: u8,

    #[serde(default = "default_nav_cc")]
    pub nav_cc: u8,
}

fn default_device_name() -> String {
    "MPK mini".to_string()
}

fn default_target_process() -> String {
    "Resolve.exe".to_string()
}

fn default_jog_cc() -> u8 {
    11
}

fn default_zoom_cc() -> u8 {
    12
}

fn default_nav_cc() -> u8 {
    13
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device_name: default_device_name(),
            target_process: default_target_process(),
            jog_cc: default_jog_cc(),
            zoom_cc: default_zoom_cc(),
            nav_cc: default_nav_cc(),
        }
    }
}

impl AppConfig {
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        if path_ref.exists() {
            let content = fs::read_to_string(path_ref)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = AppConfig::default();
            let serialized = toml::to_string_pretty(&config)?;
            fs::write(path_ref, serialized)?;
            Ok(config)
        }
    }
}
