use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcut: String,
    pub images_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcut: "ctrl+alt+w".to_string(),
            images_path: String::new(),
        }
    }
}

pub fn get_default_images_dir<R: tauri::Runtime>(app: &impl Manager<R>) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("images");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_config_path<R: tauri::Runtime>(app: &impl Manager<R>) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Error obtaining config directory: {}", e))?;
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Error creating config directory: {}", e))?;
    }
    Ok(dir.join("config.json"))
}

pub fn load_config<R: tauri::Runtime>(app: &impl Manager<R>) -> AppConfig {
    let default_images_dir = get_default_images_dir(app);
    let default_images_str = default_images_dir.to_string_lossy().to_string();

    if let Ok(path) = get_config_path(app) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                    if config.images_path.trim().is_empty() {
                        config.images_path = default_images_str;
                        let _ = save_config(app, &config);
                    }
                    return config;
                }
            }
        } else {
            let mut default_config = AppConfig::default();
            default_config.images_path = default_images_str;
            let _ = save_config(app, &default_config);
            return default_config;
        }
    }

    let mut fallback = AppConfig::default();
    fallback.images_path = default_images_str;
    fallback
}

pub fn save_config<R: tauri::Runtime>(
    app: &impl Manager<R>,
    config: &AppConfig,
) -> Result<(), String> {
    let path = get_config_path(app)?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Error serializing configuration: {}", e))?;
    fs::write(path, json).map_err(|e| format!("Error writing configuration file: {}", e))?;
    Ok(())
}
