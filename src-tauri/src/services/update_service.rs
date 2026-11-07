//! Checks GitHub Releases for a newer version of Easy Paper.
//!
//! Uses the public, unauthenticated GitHub API (60 requests/hour per IP), so it
//! runs once when the app starts plus whenever the user asks. Drafts and
//! pre-releases are ignored by the `releases/latest` endpoint, so a version can
//! be tested as a pre-release without notifying anyone.

use std::sync::Mutex;

use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::models::UpdateInfo;
use crate::services::http_client::AGENT;

const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/LucasOSantana/easy-paper/releases/latest";
const RELEASES_PREFIX: &str = "https://github.com/LucasOSantana/easy-paper/releases/";

/// Event sent to open windows when the startup check finishes.
pub const UPDATE_EVENT: &str = "update-status";

/// Result of the most recent successful check.
pub struct UpdateState(pub Mutex<Option<UpdateInfo>>);

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    assets: Vec<ReleaseAsset>,
}

#[derive(Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

/// Runs the startup check in the background. Failures (offline, rate limit)
/// are only logged: the user can still check manually.
pub fn check_on_startup(app: AppHandle) {
    std::thread::spawn(move || match check_for_updates(&app) {
        Ok(info) => {
            let _ = app.emit(UPDATE_EVENT, &info);
        }
        Err(err) => eprintln!("[update] Startup check failed: {}", err),
    });
}

/// Fetches the latest release, caches the result and returns it.
pub fn check_for_updates(app: &AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app.package_info().version.to_string();
    let release = fetch_latest_release()?;
    let info = build_update_info(&current_version, release);

    if let Some(state) = app.try_state::<UpdateState>() {
        *state.0.lock().unwrap() = Some(info.clone());
    }
    Ok(info)
}

fn fetch_latest_release() -> Result<Release, String> {
    AGENT
        .get(LATEST_RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .call()
        .map_err(describe_error)?
        .body_mut()
        .read_json()
        .map_err(describe_error)
}

fn build_update_info(current_version: &str, release: Release) -> UpdateInfo {
    let latest_version = release.tag_name.trim().trim_start_matches(['v', 'V']).to_string();
    let update_available = is_newer(&latest_version, current_version);

    // Only trust links that point to this repository's releases.
    let release_url = if release.html_url.starts_with(RELEASES_PREFIX) {
        release.html_url
    } else {
        format!("{RELEASES_PREFIX}latest")
    };

    let installer = |extension: &str| {
        release
            .assets
            .iter()
            .find(|asset| {
                asset.name.to_ascii_lowercase().ends_with(extension)
                    && asset.browser_download_url.starts_with(RELEASES_PREFIX)
            })
            .map(|asset| asset.browser_download_url.clone())
    };

    UpdateInfo {
        current_version: current_version.to_string(),
        latest_version,
        update_available,
        release_url,
        download_url: installer(".msi").or_else(|| installer(".exe")),
    }
}

/// Compares dotted numeric versions ("1.10.0" > "1.9.2"). Any suffix such as
/// "-beta.1" is ignored; missing parts count as zero.
fn is_newer(candidate: &str, current: &str) -> bool {
    fn parts(version: &str) -> Vec<u64> {
        version
            .split(['-', '+'])
            .next()
            .unwrap_or_default()
            .split('.')
            .map(|part| part.trim().parse().unwrap_or(0))
            .collect()
    }

    let (a, b) = (parts(candidate), parts(current));
    let len = a.len().max(b.len());
    let at = |v: &Vec<u64>, i: usize| v.get(i).copied().unwrap_or(0);

    for i in 0..len {
        match at(&a, i).cmp(&at(&b, i)) {
            std::cmp::Ordering::Greater => return true,
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Equal => {}
        }
    }
    false
}

fn describe_error(error: ureq::Error) -> String {
    match error {
        ureq::Error::StatusCode(404) => "No releases have been published yet.".to_string(),
        ureq::Error::StatusCode(403) | ureq::Error::StatusCode(429) => {
            "GitHub's request limit was reached. Try again in a while.".to_string()
        }
        error => format!("Could not reach GitHub: {}", error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("1.3.0", "1.2.0"));
        assert!(is_newer("1.10.0", "1.9.9"));
        assert!(is_newer("2.0", "1.99.99"));
        assert!(!is_newer("1.2.0", "1.2.0"));
        assert!(!is_newer("1.2", "1.2.0"));
        assert!(!is_newer("1.1.9", "1.2.0"));
        assert!(!is_newer("1.2.0-beta.1", "1.2.0"));
    }

    fn release(tag: &str, assets: &[(&str, &str)]) -> Release {
        Release {
            tag_name: tag.to_string(),
            html_url: format!("{RELEASES_PREFIX}tag/{tag}"),
            assets: assets
                .iter()
                .map(|(name, url)| ReleaseAsset {
                    name: name.to_string(),
                    browser_download_url: url.to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn picks_the_msi_from_this_repository() {
        let msi = format!("{RELEASES_PREFIX}download/v1.3.0/Easy.Paper_1.3.0_x64_en-US.msi");
        let info = build_update_info(
            "1.2.0",
            release("v1.3.0", &[("notes.txt", "https://example.com/n"), ("Easy.Paper_1.3.0_x64_en-US.msi", &msi)]),
        );

        assert!(info.update_available);
        assert_eq!(info.latest_version, "1.3.0");
        assert_eq!(info.download_url.as_deref(), Some(msi.as_str()));
    }

    #[test]
    fn ignores_installers_hosted_elsewhere() {
        let info = build_update_info("1.2.0", release("v1.3.0", &[("evil.msi", "https://example.com/evil.msi")]));
        assert_eq!(info.download_url, None);
    }

    #[test]
    fn same_version_is_not_an_update() {
        assert!(!build_update_info("1.2.0", release("v1.2.0", &[])).update_available);
    }
}
