//! First-run backup of the user's original wallpaper.
//!
//! The first time Easy Paper runs, before it changes anything, the image on
//! each monitor is copied into the wallpaper folder so the user can go back to
//! it from the carousel. Two guards keep this from creating duplicates:
//! - it runs once: `original_wallpaper_saved` is stored in the config after
//!   the first attempt, even when there was nothing to save;
//! - it compares by content: an image already in the folder (or already saved
//!   for another monitor) is not copied again.

use std::fs;
use std::path::{Path, PathBuf};

use image::ImageFormat;
use tauri::{AppHandle, Manager};

use crate::models::AppConfig;
use crate::services::config_service::{get_images_directory, save_config};
use crate::services::thumbnail_service::list_image_files;
use crate::services::wallpaper_service::current_wallpapers;
use crate::state::ConfigState;

const BACKUP_NAME: &str = "Original wallpaper";

pub fn save_original_wallpaper_once(app: &AppHandle) {
    let config_state = app.state::<ConfigState>();
    if config_state.0.lock().unwrap().original_wallpaper_saved {
        return;
    }

    let folder = get_images_directory(app);
    match backup_current_wallpapers(&folder) {
        Ok(saved) => {
            for path in saved {
                println!("[backup] Saved original wallpaper to {}", path.display());
            }
        }
        Err(err) => eprintln!("[backup] Could not save the original wallpaper: {}", err),
    }

    // Mark as done whatever happened: from now on the wallpaper on screen may
    // be one chosen in Easy Paper, which must not be "backed up" again.
    let mut config = config_state.0.lock().unwrap();
    let updated = AppConfig {
        original_wallpaper_saved: true,
        ..config.clone()
    };
    if let Err(err) = save_config(app, &updated) {
        eprintln!("[backup] Could not record the backup in the config: {}", err);
    }
    *config = updated;
}

/// Copies the image of every monitor into `folder`, skipping images whose
/// content is already there. Returns the files created.
fn backup_current_wallpapers(folder: &Path) -> Result<Vec<PathBuf>, String> {
    fs::create_dir_all(folder).map_err(|e| format!("Cannot create {}: {}", folder.display(), e))?;

    let wallpapers = current_wallpapers();
    let multiple = wallpapers.len() > 1;
    let mut known_files = list_image_files(folder);
    let mut saved = Vec::new();

    for wallpaper in wallpapers {
        let Some(source) = resolve_source(&wallpaper.path, wallpaper.monitor) else {
            continue;
        };
        let Ok(bytes) = fs::read(&source) else {
            continue;
        };
        let Some(extension) = extension_for(&bytes) else {
            continue;
        };
        if find_same_content(&bytes, &known_files).is_some() {
            continue;
        }

        let stem = if multiple {
            format!("{} (monitor {})", BACKUP_NAME, wallpaper.monitor)
        } else {
            BACKUP_NAME.to_string()
        };
        let target = unique_path(folder, &stem, extension);

        fs::write(&target, &bytes)
            .map_err(|e| format!("Cannot write {}: {}", target.display(), e))?;

        known_files.push(target.clone());
        saved.push(target);
    }

    Ok(saved)
}

/// The path Windows reports may point to a file that no longer exists (for
/// example a downloaded image that was deleted). Windows keeps its own copy
/// of the wallpaper under `%APPDATA%\Microsoft\Windows\Themes`; use it then.
fn resolve_source(reported: &Path, monitor: u32) -> Option<PathBuf> {
    if reported.is_file() {
        return Some(reported.to_path_buf());
    }

    let themes = PathBuf::from(std::env::var_os("APPDATA")?)
        .join("Microsoft")
        .join("Windows")
        .join("Themes");

    [
        themes.join(format!("Transcoded_{:03}", monitor.saturating_sub(1))),
        themes.join("TranscodedWallpaper"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
}

/// File extension for the image formats the carousel can show, detected from
/// the content (Windows' own copies have no extension).
fn extension_for(bytes: &[u8]) -> Option<&'static str> {
    match image::guess_format(bytes).ok()? {
        ImageFormat::Jpeg => Some("jpg"),
        ImageFormat::Png => Some("png"),
        ImageFormat::WebP => Some("webp"),
        ImageFormat::Bmp => Some("bmp"),
        _ => None,
    }
}

/// A file in `candidates` with exactly these bytes. Only files of the same
/// size are read.
fn find_same_content<'a>(bytes: &[u8], candidates: &'a [PathBuf]) -> Option<&'a PathBuf> {
    candidates.iter().find(|candidate| {
        fs::metadata(candidate).map(|meta| meta.len() == bytes.len() as u64).unwrap_or(false)
            && fs::read(candidate).map(|other| other == bytes).unwrap_or(false)
    })
}

/// `folder/stem.ext`, or `folder/stem 2.ext`, `stem 3.ext`… if taken.
fn unique_path(folder: &Path, stem: &str, extension: &str) -> PathBuf {
    let first = folder.join(format!("{stem}.{extension}"));
    if !first.exists() {
        return first;
    }

    (2..)
        .map(|n| folder.join(format!("{stem} {n}.{extension}")))
        .find(|candidate| !candidate.exists())
        .expect("an unused file name always exists")
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG_HEADER: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0];
    const JPEG_HEADER: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0x10];

    fn temp_folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("easy-paper-backup-test-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn detects_extension_from_content() {
        assert_eq!(extension_for(PNG_HEADER), Some("png"));
        assert_eq!(extension_for(JPEG_HEADER), Some("jpg"));
        assert_eq!(extension_for(b"not an image"), None);
    }

    #[test]
    fn finds_existing_copy_by_content_not_name() {
        let dir = temp_folder("content");
        let existing = dir.join("some other name.jpg");
        fs::write(&existing, JPEG_HEADER).unwrap();
        let different = dir.join("different.jpg");
        fs::write(&different, [JPEG_HEADER, b"x"].concat()).unwrap();

        let files = vec![different.clone(), existing.clone()];
        assert_eq!(find_same_content(JPEG_HEADER, &files), Some(&existing));
        assert_eq!(find_same_content(PNG_HEADER, &files), None);
    }

    #[test]
    fn never_overwrites_an_existing_file() {
        let dir = temp_folder("names");
        assert_eq!(unique_path(&dir, BACKUP_NAME, "jpg"), dir.join("Original wallpaper.jpg"));

        fs::write(dir.join("Original wallpaper.jpg"), b"a").unwrap();
        assert_eq!(unique_path(&dir, BACKUP_NAME, "jpg"), dir.join("Original wallpaper 2.jpg"));
    }
}
