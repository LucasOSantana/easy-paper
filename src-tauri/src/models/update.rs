use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    /// Release page on GitHub (notes, all assets).
    pub release_url: String,
    /// Release notes from `latest.json`, when there are any.
    pub notes: Option<String>,
}
