//! Photo sources that need the user's own API key (Unsplash, Pexels),
//! normalized to a single shape for the frontend.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StockProvider {
    Unsplash,
    Pexels,
}

impl StockProvider {
    pub fn id(self) -> &'static str {
        match self {
            Self::Unsplash => "unsplash",
            Self::Pexels => "pexels",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Unsplash => "Unsplash",
            Self::Pexels => "Pexels",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StockSearch {
    pub provider: StockProvider,
    pub query: String,
    /// "landscape", "portrait" or "" for any.
    pub orientation: String,
    pub page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockPhoto {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub thumb_url: String,
    /// Full-size image (capped at 4K) to save into the wallpaper folder.
    pub download_url: String,
    /// The photo's page on the provider's site (for attribution).
    pub page_url: String,
    pub author_name: String,
    pub author_url: String,
    /// Unsplash only: endpoint that must be called when a photo is downloaded.
    pub download_location: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StockPage {
    pub photos: Vec<StockPhoto>,
    pub page: u32,
    pub has_next: bool,
}
