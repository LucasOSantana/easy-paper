//! In-app updates through `tauri-plugin-updater`.
//!
//! The updater reads `latest.json` from the latest GitHub release (endpoint and
//! public key live in `tauri.conf.json` under `plugins.updater`). Drafts and
//! pre-releases are not served by `releases/latest`, so a version can be tested
//! as a pre-release without notifying anyone.
//!
//! Every installer is verified against the public key before it runs. On
//! Windows the plugin starts the MSI and exits the app with
//! `std::process::exit`, which skips `RunEvent::Exit`, so the taskbar is
//! restored in `on_before_exit` instead.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::models::UpdateInfo;
use crate::services::taskbar_service::set_taskbar_transparency;

const RELEASES_PREFIX: &str = "https://github.com/LucasOSantana/easy-paper/releases/";

/// Event sent to open windows when the startup check finishes.
pub const UPDATE_EVENT: &str = "update-status";
/// Event sent while the installer is being downloaded.
pub const UPDATE_PROGRESS_EVENT: &str = "update-progress";

/// Result of the most recent successful check, plus the update itself so it
/// can be installed without checking again.
pub struct UpdateState {
    info: Mutex<Option<UpdateInfo>>,
    pending: Mutex<Option<Update>>,
    installing: Mutex<bool>,
}

impl UpdateState {
    pub fn new() -> Self {
        Self { info: Mutex::new(None), pending: Mutex::new(None), installing: Mutex::new(false) }
    }

    pub fn info(&self) -> Option<UpdateInfo> {
        self.info.lock().unwrap().clone()
    }
}

#[derive(Clone, Serialize)]
struct DownloadProgress {
    downloaded: u64,
    total: Option<u64>,
}

/// Runs the startup check in the background. Failures (offline, no release
/// yet) are only logged: the user can still check manually.
pub fn check_on_startup(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        match check_for_updates(&app).await {
            Ok(info) => {
                let _ = app.emit(UPDATE_EVENT, &info);
            }
            Err(err) => eprintln!("[update] Startup check failed: {}", err),
        }
    });
}

/// Fetches `latest.json`, caches the result and returns it.
pub async fn check_for_updates(app: &AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app.package_info().version.to_string();
    // `install` exits the process on Windows, skipping `RunEvent::Exit`.
    let exit_app = app.clone();
    let update = app
        .updater_builder()
        .on_before_exit(move || {
            let _ = set_taskbar_transparency(&exit_app, false);
        })
        .build()
        .map_err(|e| crate::tr!(
            en: format!("The updater is not configured: {}", e),
            pt: format!("O atualizador não está configurado: {}", e),
        ))?
        .check()
        .await
        .map_err(describe_error)?;

    let info = match &update {
        Some(update) => UpdateInfo {
            current_version,
            latest_version: update.version.clone(),
            update_available: true,
            release_url: release_url(&update.version),
            notes: update.body.clone().filter(|notes| !notes.trim().is_empty()),
        },
        None => UpdateInfo {
            latest_version: current_version.clone(),
            release_url: format!("{RELEASES_PREFIX}latest"),
            current_version,
            update_available: false,
            notes: None,
        },
    };

    let state = app.state::<UpdateState>();
    *state.info.lock().unwrap() = Some(info.clone());
    *state.pending.lock().unwrap() = update;
    Ok(info)
}

/// Downloads the pending update, verifies its signature and runs the
/// installer. On Windows this does not return on success: the app exits and
/// the installer restarts it once it is done.
pub async fn install_update(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<UpdateState>();
    {
        let mut installing = state.installing.lock().unwrap();
        if *installing {
            return Err(crate::tr!(
                en: "The update is already being installed.",
                pt: "A atualização já está sendo instalada.",
            ));
        }
        *installing = true;
    }

    let result = download_and_install(app).await;
    *state.installing.lock().unwrap() = false;
    result
}

async fn download_and_install(app: &AppHandle) -> Result<(), String> {
    // The cached update came from a check that may be hours old: check again
    // so we never install something that was since replaced.
    check_for_updates(app).await?;
    let update = app
        .state::<UpdateState>()
        .pending
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| crate::tr!(en: "You're already on the latest version.", pt: "Você já está na versão mais recente."))?;

    let mut downloaded = 0u64;
    let bytes = update
        .download(
            |chunk, total| {
                downloaded += chunk as u64;
                let _ = app.emit(UPDATE_PROGRESS_EVENT, DownloadProgress { downloaded, total });
            },
            || {},
        )
        .await
        .map_err(|e| crate::tr!(
            en: format!("Could not download the update: {}", e),
            pt: format!("Não foi possível baixar a atualização: {}", e),
        ))?;

    update.install(bytes).map_err(|e| crate::tr!(
        en: format!("Could not install the update: {}", e),
        pt: format!("Não foi possível instalar a atualização: {}", e),
    ))
}

fn release_url(version: &str) -> String {
    format!("{RELEASES_PREFIX}tag/v{}", version.trim_start_matches(['v', 'V']))
}

fn describe_error(error: tauri_plugin_updater::Error) -> String {
    let message = error.to_string();
    if message.contains("404") {
        crate::tr!(
            en: "No update information was published for the latest release yet.",
            pt: "Ainda não há informações de atualização publicadas para a versão mais recente.",
        )
    } else if message.contains("403") || message.contains("429") {
        crate::tr!(
            en: "GitHub's request limit was reached. Try again in a while.",
            pt: "O limite de pedidos do GitHub foi atingido. Tente de novo daqui a pouco.",
        )
    } else {
        crate::tr!(
            en: format!("Could not check for updates: {}", message),
            pt: format!("Não foi possível procurar atualizações: {}", message),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_url_points_to_the_tag() {
        assert_eq!(release_url("1.4.0"), format!("{RELEASES_PREFIX}tag/v1.4.0"));
        assert_eq!(release_url("v1.4.0"), format!("{RELEASES_PREFIX}tag/v1.4.0"));
    }
}
