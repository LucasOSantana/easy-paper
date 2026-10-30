use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct WallhavenSearch {
    pub query: String,
    pub sorting: String,
    pub ratios: String,
    pub match_screen: bool,
    pub page: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WallhavenPage {
    pub data: Vec<WallhavenWallpaper>,
    pub meta: WallhavenMeta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WallhavenWallpaper {
    pub id: String,
    pub path: String,
    pub resolution: String,
    pub thumbs: WallhavenThumbs,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WallhavenThumbs {
    pub small: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WallhavenMeta {
    pub current_page: u32,
    pub last_page: u32,
}
