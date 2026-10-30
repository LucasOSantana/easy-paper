use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcut: String,
    pub images_path: String,
    #[serde(default)]
    pub taskbar_transparent: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcut: "super+j".to_string(),
            images_path: String::new(),
            taskbar_transparent: false,
        }
    }
}
