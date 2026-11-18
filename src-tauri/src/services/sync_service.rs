//! Backup and sync.
//!
//! - **Backup**: exports the synced settings (shortcut, taskbar, rotation,
//!   favorites) to `easy-paper-backup.json`, optionally with a copy of every
//!   wallpaper in a `wallpapers` folder next to it. Importing restores the
//!   settings and copies back the images that are missing.
//! - **Sync**: keeps `easy-paper-sync.json` in a folder the user picks,
//!   usually inside OneDrive, Dropbox or Google Drive, which carries it to
//!   their other PCs. Whole-file "last change wins": the side with the newer
//!   `settings_updated_at` is copied over the other. Images are not synced
//!   by the app (keep the wallpaper folder inside the cloud folder for that).

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::models::{
    AppConfig, BackupSummary, SyncDocument, SyncFolderInfo, SyncStatus, SyncedSettings,
};
use crate::services::config_service::{
    get_images_directory, modify_config, modify_config_with_stamp, now_ms, SyncStamp,
};
use crate::services::rotation_service::reset_rotation_timer;
use crate::services::taskbar_service::set_taskbar_transparency;
use crate::services::thumbnail_service::list_image_files;
use crate::state::ConfigState;

pub const SYNC_FILE_NAME: &str = "easy-paper-sync.json";
pub const BACKUP_FILE_NAME: &str = "easy-paper-backup.json";
pub const BACKUP_IMAGES_DIR: &str = "wallpapers";

pub const SYNC_STATUS_EVENT: &str = "sync-status";

/// How often the sync folder is checked for changes made on another PC.
const SYNC_INTERVAL: Duration = Duration::from_secs(60);
const SYNC_STARTUP_DELAY: Duration = Duration::from_secs(5);
/// The periodic check skips unchanged files, but still reads the file every
/// this many checks (~15 min) in case a cloud client kept its timestamp.
const FULL_CHECK_EVERY: u32 = 15;

/// What both sides looked like after the last successful sync. While it is
/// the same, nothing can have changed and the file does not need to be read.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Fingerprint {
    file: PathBuf,
    /// Modification time and size of the sync file (`None` when missing).
    remote: Option<(SystemTime, u64)>,
    local_updated_at: u64,
}

fn fingerprint(file: &Path, local_updated_at: u64) -> Fingerprint {
    // Metadata only: does not download an online-only cloud file.
    let remote = fs::metadata(file)
        .ok()
        .and_then(|meta| meta.modified().ok().map(|modified| (modified, meta.len())));
    Fingerprint {
        file: file.to_path_buf(),
        remote,
        local_updated_at,
    }
}

/// How thorough a sync run is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncMode {
    /// "Sync now", after local changes, and every few periodic checks.
    Full,
    /// Periodic check: skip when the fingerprint did not change.
    Quick,
}

pub struct SyncState {
    status: Mutex<SyncStatus>,
    /// Serializes sync runs (worker, "Sync now" and pushes after changes)
    /// and holds the fingerprint of the last successful one.
    running: Mutex<Option<Fingerprint>>,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(SyncStatus::default()),
            running: Mutex::new(None),
        }
    }
}

pub fn sync_status<R: Runtime>(app: &AppHandle<R>) -> SyncStatus {
    app.state::<SyncState>().status.lock().unwrap().clone()
}

/// Updates the status and notifies the windows only when something they
/// show changed. A new check time alone is not announced: Settings reads it
/// with `get_sync_status` while it is open.
fn set_status<R: Runtime>(app: &AppHandle<R>, change: impl FnOnce(&mut SyncStatus)) {
    let (status, changed) = {
        let state = app.state::<SyncState>();
        let mut status = state.status.lock().unwrap();
        let before = status.clone();
        change(&mut status);
        (status.clone(), !same_except_check_time(&before, &status))
    };
    if changed {
        let _ = app.emit(SYNC_STATUS_EVENT, &status);
    }
}

fn same_except_check_time(a: &SyncStatus, b: &SyncStatus) -> bool {
    a.last_action == b.last_action
        && a.remote_device == b.remote_device
        && a.last_error == b.last_error
        && a.last_checked_at.is_some() == b.last_checked_at.is_some()
}

fn device_name() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| crate::tr!(en: "Unknown PC", pt: "PC desconhecido"))
}

fn current_config<R: Runtime>(app: &AppHandle<R>) -> AppConfig {
    app.state::<ConfigState>().0.lock().unwrap().clone()
}

/// Writes through a temporary file, so a cloud client never uploads a
/// half-written document.
fn write_document(path: &Path, document: &SyncDocument) -> Result<(), String> {
    let json = serde_json::to_string_pretty(document)
        .map_err(|e| crate::tr!(
            en: format!("Could not prepare the file: {}", e),
            pt: format!("Não foi possível preparar o arquivo: {}", e),
        ))?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, json).map_err(|e| could_not_write(&temp, e))?;
    fs::rename(&temp, path).map_err(|e| {
        let _ = fs::remove_file(&temp);
        could_not_write(path, e)
    })
}

fn could_not_write(path: &Path, error: std::io::Error) -> String {
    crate::tr!(
        en: format!("Could not write {}: {}", path.display(), error),
        pt: format!("Não foi possível gravar {}: {}", path.display(), error),
    )
}

fn read_document(path: &Path) -> Result<Option<SyncDocument>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)
        .map_err(|e| crate::tr!(
            en: format!("Could not read {}: {}", path.display(), e),
            pt: format!("Não foi possível ler {}: {}", path.display(), e),
        ))?;
    SyncDocument::parse(&content).map(Some)
}

fn sync_folder_path(folder: &str) -> Result<PathBuf, String> {
    let folder = folder.trim();
    if folder.is_empty() {
        return Err(crate::tr!(en: "Sync is off.", pt: "A sincronização está desligada."));
    }
    let path = PathBuf::from(folder);
    if !path.is_dir() {
        return Err(crate::tr!(
            en: format!("The sync folder was not found: {}", folder),
            pt: format!("A pasta de sincronização não foi encontrada: {}", folder),
        ));
    }
    Ok(path)
}

// --------------------------------------------------------------------------
// Applying settings from a backup or another PC
// --------------------------------------------------------------------------

/// Replaces the synced settings with `incoming`, registering the shortcut and
/// updating the taskbar. Returns problems that did not stop the restore.
pub fn apply_synced_settings<R: Runtime>(
    app: &AppHandle<R>,
    incoming: SyncedSettings,
    stamp: SyncStamp,
) -> Result<Vec<String>, String> {
    let current = current_config(app);
    let mut warnings = Vec::new();

    let shortcut = incoming.shortcut.trim().to_lowercase();
    let mut shortcut_to_save = current.shortcut.clone();
    if shortcut != current.shortcut {
        match register_shortcut(app, &shortcut, &current.shortcut) {
            Ok(()) => shortcut_to_save = shortcut,
            Err(err) => warnings.push(err),
        }
    }

    let mut taskbar_to_save = current.taskbar_transparent;
    if incoming.taskbar_transparent != current.taskbar_transparent {
        match set_taskbar_transparency(app, incoming.taskbar_transparent) {
            Ok(()) => taskbar_to_save = incoming.taskbar_transparent,
            Err(err) => warnings.push(crate::tr!(
                en: format!("Taskbar: {}", err),
                pt: format!("Barra de tarefas: {}", err),
            )),
        }
    }

    modify_config_with_stamp(app, stamp, |config| {
        config.shortcut = shortcut_to_save;
        config.taskbar_transparent = taskbar_to_save;
        config.auto_rotate = incoming.auto_rotate;
        config.favorites = incoming.favorites;
        if crate::i18n::SUPPORTED_SETTINGS.contains(&incoming.language.as_str()) {
            config.language = incoming.language;
        }
        Ok(())
    })?;
    reset_rotation_timer(app);

    Ok(warnings)
}

fn register_shortcut<R: Runtime>(app: &AppHandle<R>, shortcut: &str, previous: &str) -> Result<(), String> {
    let parsed = Shortcut::from_str(shortcut)
        .map_err(|_| crate::tr!(
            en: format!("The shortcut \"{}\" is not valid and was not restored.", shortcut),
            pt: format!("O atalho \"{}\" não é válido e não foi restaurado.", shortcut),
        ))?;

    let _ = app.global_shortcut().unregister_all();
    if app.global_shortcut().register(parsed).is_err() {
        if let Ok(old) = Shortcut::from_str(previous) {
            let _ = app.global_shortcut().register(old);
        }
        return Err(crate::tr!(
            en: format!("The shortcut \"{}\" is in use by another app, so \"{}\" was kept.", shortcut, previous),
            pt: format!("O atalho \"{}\" está em uso por outro app, então \"{}\" foi mantido.", shortcut, previous),
        ));
    }
    Ok(())
}

// --------------------------------------------------------------------------
// Backup
// --------------------------------------------------------------------------

/// Copies the images of `from` whose name does not exist in `to` yet.
fn copy_missing_images(from: &Path, to: &Path) -> Result<usize, String> {
    fs::create_dir_all(to).map_err(|e| crate::tr!(
        en: format!("Could not create {}: {}", to.display(), e),
        pt: format!("Não foi possível criar {}: {}", to.display(), e),
    ))?;

    let mut copied = 0;
    for source in list_image_files(from) {
        let Some(name) = source.file_name() else { continue };
        let target = to.join(name);
        if target.exists() {
            continue;
        }
        fs::copy(&source, &target)
            .map_err(|e| crate::tr!(
                en: format!("Could not copy {}: {}", source.display(), e),
                pt: format!("Não foi possível copiar {}: {}", source.display(), e),
            ))?;
        copied += 1;
    }
    Ok(copied)
}

/// Creates `Easy Paper backup <date time>` inside `destination`.
pub fn export_backup<R: Runtime>(
    app: &AppHandle<R>,
    destination: &Path,
    include_images: bool,
) -> Result<BackupSummary, String> {
    let config = current_config(app);
    let name = crate::tr!(
        en: format!("Easy Paper backup {}", local_timestamp()),
        pt: format!("Backup do Easy Paper {}", local_timestamp()),
    );
    let folder = unique_folder(destination, &name);
    fs::create_dir_all(&folder)
        .map_err(|e| crate::tr!(
            en: format!("Could not create the backup folder: {}", e),
            pt: format!("Não foi possível criar a pasta do backup: {}", e),
        ))?;

    let document = SyncDocument::new(
        SyncedSettings::from(&config),
        config.settings_updated_at.max(1),
        device_name(),
    );
    write_document(&folder.join(BACKUP_FILE_NAME), &document)?;

    let images = if include_images {
        copy_missing_images(&get_images_directory(app), &folder.join(BACKUP_IMAGES_DIR))?
    } else {
        0
    };

    Ok(BackupSummary {
        path: folder.to_string_lossy().to_string(),
        images,
        warnings: Vec::new(),
    })
}

/// Restores a backup file. Images in a `wallpapers` folder next to it are
/// copied into the wallpaper folder, skipping names that already exist.
pub fn import_backup<R: Runtime>(app: &AppHandle<R>, file: &Path) -> Result<BackupSummary, String> {
    let document = read_document(file)?
        .ok_or_else(|| crate::tr!(en: "The backup file was not found.", pt: "O arquivo de backup não foi encontrado."))?;

    let images_dir = file.parent().map(|parent| parent.join(BACKUP_IMAGES_DIR));
    let images = match images_dir {
        Some(dir) if dir.is_dir() => copy_missing_images(&dir, &get_images_directory(app))?,
        _ => 0,
    };

    // A restore is a local change: it becomes the newest version and is
    // pushed to the sync folder when sync is on.
    let warnings = apply_synced_settings(app, document.settings, SyncStamp::Local)?;

    Ok(BackupSummary {
        path: file.to_string_lossy().to_string(),
        images,
        warnings,
    })
}

fn unique_folder(parent: &Path, name: &str) -> PathBuf {
    let mut candidate = parent.join(name);
    let mut n = 2;
    while candidate.exists() {
        candidate = parent.join(format!("{} ({})", name, n));
        n += 1;
    }
    candidate
}

#[cfg(windows)]
fn local_timestamp() -> String {
    let now = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    format!(
        "{:04}-{:02}-{:02} {:02}-{:02}",
        now.wYear, now.wMonth, now.wDay, now.wHour, now.wMinute
    )
}

#[cfg(not(windows))]
fn local_timestamp() -> String {
    now_ms().to_string()
}

// --------------------------------------------------------------------------
// Sync
// --------------------------------------------------------------------------

/// Describes the sync file already in `folder`, if any, so the user can
/// choose which side to keep before connecting.
pub fn inspect_sync_folder(folder: &str) -> Result<Option<SyncFolderInfo>, String> {
    let path = sync_folder_path(folder)?;
    Ok(read_document(&path.join(SYNC_FILE_NAME))?.map(|document| SyncFolderInfo {
        device: document.device,
        updated_at: document.updated_at,
        favorites: document.settings.favorites.len(),
    }))
}

/// Turns sync on with `folder`. With `use_folder_settings`, the settings in
/// the folder replace this PC's; otherwise this PC's replace the folder's.
pub fn connect_sync_folder<R: Runtime>(
    app: &AppHandle<R>,
    folder: &str,
    use_folder_settings: bool,
) -> Result<SyncStatus, String> {
    let path = sync_folder_path(folder)?;
    let remote = read_document(&path.join(SYNC_FILE_NAME))?;

    let mut warnings = Vec::new();
    match remote {
        Some(remote) if use_folder_settings => {
            warnings = apply_synced_settings(app, remote.settings, SyncStamp::Exact(remote.updated_at))?;
        }
        _ => {
            // Make this PC's settings the newest so the next run pushes them.
            modify_config_with_stamp(app, SyncStamp::Exact(now_ms()), |_| Ok(()))?;
        }
    }

    let folder = path.to_string_lossy().to_string();
    modify_config(app, |config| {
        config.sync.folder = folder;
        Ok(())
    })?;

    sync_now(app)?;
    if !warnings.is_empty() {
        set_status(app, |status| status.last_error = Some(warnings.join(" ")));
    }
    Ok(sync_status(app))
}

pub fn disconnect_sync<R: Runtime>(app: &AppHandle<R>) -> Result<AppConfig, String> {
    let updated = modify_config(app, |config| {
        config.sync.folder.clear();
        Ok(())
    })?;
    *app.state::<SyncState>().running.lock().unwrap() = None;
    set_status(app, |status| *status = SyncStatus::default());
    Ok(updated)
}

/// Compares this PC with the sync folder and copies the newer side over the
/// older one.
pub fn sync_now<R: Runtime>(app: &AppHandle<R>) -> Result<SyncStatus, String> {
    run_locked(app, SyncMode::Full)
}

fn run_locked<R: Runtime>(app: &AppHandle<R>, mode: SyncMode) -> Result<SyncStatus, String> {
    let state = app.state::<SyncState>();
    let mut last = state.running.lock().unwrap();

    match run_sync(app, mode, &mut last) {
        Ok(Some((action, device))) => {
            set_status(app, |status| {
                status.last_checked_at = Some(now_ms());
                status.last_action = Some(action.to_string());
                status.remote_device = device;
                status.last_error = None;
            });
            Ok(sync_status(app))
        }
        Ok(None) => {
            // Nothing changed on either side since the last sync.
            set_status(app, |status| status.last_checked_at = Some(now_ms()));
            Ok(sync_status(app))
        }
        Err(err) => {
            // Read everything again next time.
            *last = None;
            set_status(app, |status| status.last_error = Some(err.clone()));
            Err(err)
        }
    }
}

/// Returns `None` when a quick check found nothing changed.
fn run_sync<R: Runtime>(
    app: &AppHandle<R>,
    mode: SyncMode,
    last: &mut Option<Fingerprint>,
) -> Result<Option<(&'static str, Option<String>)>, String> {
    let config = current_config(app);
    let folder = sync_folder_path(&config.sync.folder)?;
    let file = folder.join(SYNC_FILE_NAME);

    if mode == SyncMode::Quick
        && last.as_ref() == Some(&fingerprint(&file, config.settings_updated_at))
    {
        return Ok(None);
    }

    let result = exchange(app, config, &file)?;
    // Taken after the exchange: it may have written the file or changed the
    // local timestamp.
    *last = Some(fingerprint(&file, current_config(app).settings_updated_at));
    Ok(Some(result))
}

/// Reads the sync file and copies the newer side over the older one.
fn exchange<R: Runtime>(
    app: &AppHandle<R>,
    mut config: AppConfig,
    file: &Path,
) -> Result<(&'static str, Option<String>), String> {
    let remote = read_document(file)?;

    match decide(config.settings_updated_at, remote.as_ref().map(|r| r.updated_at)) {
        SyncAction::Pull => {
            let remote = remote.expect("pull implies a remote file");
            let device = remote.device.clone();
            let warnings = apply_synced_settings(app, remote.settings, SyncStamp::Exact(remote.updated_at))?;
            if !warnings.is_empty() {
                return Err(warnings.join(" "));
            }
            Ok(("pulled", Some(device)))
        }
        SyncAction::UpToDate => Ok(("up_to_date", remote.map(|r| r.device))),
        SyncAction::Push => {
            if config.settings_updated_at == 0 {
                // Never-changed settings still need a real time, or another
                // PC with untouched settings would consider them equal.
                config = modify_config_with_stamp(app, SyncStamp::Exact(now_ms()), |_| Ok(()))?;
            }
            let document = SyncDocument::new(
                SyncedSettings::from(&config),
                config.settings_updated_at,
                device_name(),
            );
            write_document(file, &document)?;
            Ok(("pushed", Some(document.device)))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SyncAction {
    Push,
    Pull,
    UpToDate,
}

fn decide(local_updated_at: u64, remote_updated_at: Option<u64>) -> SyncAction {
    match remote_updated_at {
        None => SyncAction::Push,
        Some(remote) if remote > local_updated_at => SyncAction::Pull,
        Some(remote) if remote == local_updated_at => SyncAction::UpToDate,
        Some(_) => SyncAction::Push,
    }
}

/// Pushes a local change without blocking the caller.
pub fn push_in_background<R: Runtime>(app: AppHandle<R>) {
    thread::spawn(move || {
        if let Err(err) = sync_now(&app) {
            eprintln!("[sync] {}", err);
        }
    });
}

/// Checks the sync folder at startup and then every minute, to pick up
/// changes made on other PCs.
pub fn start_sync_worker<R: Runtime>(app: AppHandle<R>) {
    thread::spawn(move || {
        thread::sleep(SYNC_STARTUP_DELAY);
        let mut checks: u32 = 0;
        loop {
            let enabled = !current_config(&app).sync.folder.trim().is_empty();
            if enabled {
                let mode = if checks.is_multiple_of(FULL_CHECK_EVERY) { SyncMode::Full } else { SyncMode::Quick };
                checks = checks.wrapping_add(1);
                if let Err(err) = run_locked(&app, mode) {
                    eprintln!("[sync] {}", err);
                }
            }
            thread::sleep(SYNC_INTERVAL);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AutoRotateConfig;

    #[test]
    fn newer_side_wins() {
        assert_eq!(decide(10, None), SyncAction::Push);
        assert_eq!(decide(10, Some(20)), SyncAction::Pull);
        assert_eq!(decide(20, Some(10)), SyncAction::Push);
        assert_eq!(decide(10, Some(10)), SyncAction::UpToDate);
    }

    #[test]
    fn fingerprint_changes_with_the_file_or_local_settings() {
        let root = std::env::temp_dir().join(format!("easy-paper-fp-{}", now_ms()));
        fs::create_dir_all(&root).unwrap();
        let file = root.join(SYNC_FILE_NAME);

        let missing = fingerprint(&file, 1);
        assert_eq!(missing.remote, None);
        assert_eq!(missing, fingerprint(&file, 1));
        assert_ne!(missing, fingerprint(&file, 2), "a local change must be noticed");

        fs::write(&file, b"{}").unwrap();
        let written = fingerprint(&file, 1);
        assert_ne!(written, missing, "a new file must be noticed");

        fs::write(&file, b"{\"longer\": true}").unwrap();
        assert_ne!(fingerprint(&file, 1), written, "a rewritten file must be noticed");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn check_time_alone_is_not_announced() {
        let before = SyncStatus {
            last_checked_at: Some(1),
            last_action: Some("up_to_date".into()),
            ..Default::default()
        };
        let mut after = before.clone();
        after.last_checked_at = Some(2);
        assert!(same_except_check_time(&before, &after));

        after.last_action = Some("pulled".into());
        assert!(!same_except_check_time(&before, &after));

        assert!(!same_except_check_time(&SyncStatus::default(), &before), "first check is announced");
    }

    #[test]
    fn document_round_trip_and_validation() {
        let settings = SyncedSettings {
            shortcut: "super+j".into(),
            taskbar_transparent: true,
            auto_rotate: AutoRotateConfig::default(),
            favorites: vec!["a.jpg".into()],
            language: "system".into(),
        };
        let document = SyncDocument::new(settings.clone(), 42, "PC".into());
        let json = serde_json::to_string(&document).unwrap();
        let parsed = SyncDocument::parse(&json).unwrap();
        assert_eq!(parsed.settings, settings);
        assert_eq!(parsed.updated_at, 42);

        assert!(SyncDocument::parse("{\"hello\": 1}").is_err());
        let foreign = json.replace("\"easy-paper\"", "\"other-app\"");
        assert!(SyncDocument::parse(&foreign).is_err());
        let future = json.replace("\"version\":1", "\"version\":99");
        assert!(SyncDocument::parse(&future).is_err());
    }

    #[test]
    fn copy_missing_images_skips_existing_names() {
        let root = std::env::temp_dir().join(format!("easy-paper-test-{}", now_ms()));
        let from = root.join("from");
        let to = root.join("to");
        fs::create_dir_all(&from).unwrap();
        fs::create_dir_all(&to).unwrap();
        fs::write(from.join("a.jpg"), b"new").unwrap();
        fs::write(from.join("b.png"), b"new").unwrap();
        fs::write(from.join("notes.txt"), b"ignored").unwrap();
        fs::write(to.join("a.jpg"), b"old").unwrap();

        assert_eq!(copy_missing_images(&from, &to).unwrap(), 1);
        assert_eq!(fs::read(to.join("a.jpg")).unwrap(), b"old");
        assert!(to.join("b.png").exists());
        assert!(!to.join("notes.txt").exists());

        let _ = fs::remove_dir_all(root);
    }
}
