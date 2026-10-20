use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WallpaperImageItem {
    pub name: String,
    pub path: String,
    pub thumbnail: String,
}
