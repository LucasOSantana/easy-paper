use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    /// Release page on GitHub (notes, all assets).
    pub release_url: String,
    /// Direct link to the Windows installer, when the release has one.
    pub download_url: Option<String>,
}
