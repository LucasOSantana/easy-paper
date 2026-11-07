use serde::{Deserialize, Serialize};

/// Shortest and longest interval accepted for automatic rotation, in minutes.
pub const MIN_ROTATION_INTERVAL_MINUTES: u32 = 5;
pub const MAX_ROTATION_INTERVAL_MINUTES: u32 = 7 * 24 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationTrigger {
    /// Change once every time Windows starts.
    OnStartup,
    /// Change every `interval_minutes` while the PC is on.
    Interval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationOrder {
    /// Go through every wallpaper in the folder before repeating any.
    Shuffle,
    /// Pick any wallpaper each time (only avoids the one currently shown).
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoRotateConfig {
    pub enabled: bool,
    pub trigger: RotationTrigger,
    pub interval_minutes: u32,
    pub order: RotationOrder,
}

impl Default for AutoRotateConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trigger: RotationTrigger::Interval,
            interval_minutes: 180,
            order: RotationOrder::Shuffle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcut: String,
    pub images_path: String,
    #[serde(default)]
    pub taskbar_transparent: bool,
    #[serde(default)]
    pub auto_rotate: AutoRotateConfig,
    /// Set after the first-run backup of the user's original wallpaper was
    /// attempted, so it never runs again.
    #[serde(default)]
    pub original_wallpaper_saved: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcut: "super+j".to_string(),
            images_path: String::new(),
            taskbar_transparent: false,
            auto_rotate: AutoRotateConfig::default(),
            original_wallpaper_saved: false,
        }
    }
}
