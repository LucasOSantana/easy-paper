//! Automatic wallpaper rotation.
//!
//! A single background thread drives both modes:
//! - `OnStartup`: changes the wallpaper once when Windows launches the app at
//!   sign-in (after turning the PC on or restarting). Relaunching the app by
//!   hand does not count. Detection relies on the `--autostart` argument that
//!   the autostart registry entry passes, because Windows Fast Startup keeps
//!   the system uptime running across shutdowns.
//! - `Interval`: changes it every N minutes of wall-clock time. Time spent
//!   asleep/hibernated counts too, so if the interval passed while the PC
//!   slept, the wallpaper changes right after it wakes. Shutting down or
//!   closing the app restarts the count (it is kept in memory only).
//!
//! Each active monitor receives a different image whenever the folder has
//! enough of them.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Manager};

use crate::models::{AutoRotateConfig, RotationOrder, RotationTrigger};
use crate::services::config_service::get_images_directory;
use crate::services::thumbnail_service::list_image_files;
use crate::services::wallpaper_service::apply_to_each_monitor;
use crate::state::ConfigState;

/// How often the worker wakes up to check the timer.
const TICK: Duration = Duration::from_secs(15);
/// Startup rotation waits for the desktop to be ready, then retries.
const STARTUP_DELAY: Duration = Duration::from_secs(5);
const STARTUP_RETRY_DELAY: Duration = Duration::from_secs(10);
const STARTUP_ATTEMPTS: u32 = 3;

#[derive(Default)]
pub struct RotationRuntime {
    /// Wall-clock time since the last change (interval mode), sleep included.
    elapsed: Duration,
    /// Remaining images of the current shuffle cycle.
    queue: Vec<PathBuf>,
    /// Folder the queue was built from; a different folder resets it.
    queue_folder: PathBuf,
}

pub struct RotationState(pub Mutex<RotationRuntime>);

impl RotationState {
    pub fn new() -> Self {
        Self(Mutex::new(RotationRuntime::default()))
    }
}

/// Restarts the interval count. Called when settings change and when the user
/// picks a wallpaper manually, so the next automatic change is a full
/// interval away.
pub fn reset_rotation_timer<R: tauri::Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<RotationState>() {
        state.0.lock().unwrap().elapsed = Duration::ZERO;
    }
}

/// Argument added by the autostart registry entry.
pub const AUTOSTART_ARG: &str = "--autostart";

pub fn start_rotation_worker(app: AppHandle, launched_at_sign_in: bool) {
    thread::spawn(move || {
        if launched_at_sign_in {
            let config = current_rotation_config(&app);
            if config.enabled && config.trigger == RotationTrigger::OnStartup {
                rotate_on_startup(&app, config.order);
            }
        }

        let mut last_tick = SystemTime::now();

        loop {
            thread::sleep(TICK);

            // Measured with the wall clock, so time the PC spent asleep is
            // included (the thread itself does not run while suspended).
            let now = SystemTime::now();
            let gap = now.duration_since(last_tick).unwrap_or(Duration::ZERO);
            last_tick = now;

            let config = current_rotation_config(&app);
            let state = app.state::<RotationState>();

            let due = {
                let mut runtime = state.0.lock().unwrap();

                if !config.enabled || config.trigger != RotationTrigger::Interval {
                    runtime.elapsed = Duration::ZERO;
                    false
                } else {
                    let (elapsed, due) = advance_timer(runtime.elapsed, gap, interval_of(&config));
                    runtime.elapsed = elapsed;
                    due
                }
            };

            if due {
                if let Err(err) = rotate_now(&app, config.order) {
                    eprintln!("[rotation] Automatic wallpaper change failed, retrying: {}", err);
                    // E.g. the desktop is not ready right after waking up:
                    // keep the change due so the next tick tries again.
                    state.0.lock().unwrap().elapsed = interval_of(&config);
                }
            }
        }
    });
}

fn interval_of(config: &AutoRotateConfig) -> Duration {
    Duration::from_secs(u64::from(config.interval_minutes) * 60)
}

/// Adds `gap` (which includes any time asleep) to the count. Returns the new
/// count and whether a change is due; the count restarts when it is.
fn advance_timer(elapsed: Duration, gap: Duration, interval: Duration) -> (Duration, bool) {
    let elapsed = elapsed + gap;
    if elapsed >= interval {
        (Duration::ZERO, true)
    } else {
        (elapsed, false)
    }
}

fn current_rotation_config(app: &AppHandle) -> AutoRotateConfig {
    app.state::<ConfigState>().0.lock().unwrap().auto_rotate.clone()
}

fn rotate_on_startup(app: &AppHandle, order: RotationOrder) {
    thread::sleep(STARTUP_DELAY);

    for attempt in 1..=STARTUP_ATTEMPTS {
        match rotate_now(app, order) {
            Ok(_) => return,
            Err(err) => {
                eprintln!("[rotation] Startup change attempt {} failed: {}", attempt, err);
                if attempt < STARTUP_ATTEMPTS {
                    thread::sleep(STARTUP_RETRY_DELAY);
                }
            }
        }
    }
}

/// Picks and applies a new wallpaper on every active monitor.
/// Returns how many monitors were changed (0 when the folder is empty).
pub fn rotate_now(app: &AppHandle, order: RotationOrder) -> Result<usize, String> {
    let folder = get_images_directory(app);
    let files = list_image_files(&folder);
    if files.is_empty() {
        return Ok(0);
    }

    let state = app.state::<RotationState>();
    let mut runtime = state.0.lock().unwrap();

    if runtime.queue_folder != folder {
        runtime.queue.clear();
        runtime.queue_folder = folder.clone();
    }

    // Drop queued images that were deleted from the folder.
    let available: Vec<String> = files.iter().map(|path| path_key(path)).collect();
    runtime.queue.retain(|path| available.contains(&path_key(path)));

    let mut used: Vec<String> = Vec::new();
    let mut choose = |current: Option<&Path>| {
        let current_key = current.map(path_key);
        let pick = match order {
            RotationOrder::Shuffle => pick_shuffled(&mut runtime, &files, current_key.as_deref(), &used),
            RotationOrder::Random => pick_random(&files, current_key.as_deref(), &used),
        };
        if let Some(path) = &pick {
            used.push(path_key(path));
        }
        pick
    };

    apply_to_each_monitor(&mut choose).map_err(|err| err.to_string())
}

/// Next image of the current shuffle cycle, skipping the image already on
/// this monitor and the ones just given to other monitors.
fn pick_shuffled(
    runtime: &mut RotationRuntime,
    files: &[PathBuf],
    current: Option<&str>,
    used: &[String],
) -> Option<PathBuf> {
    for _ in 0..2 {
        if runtime.queue.is_empty() {
            let mut cycle = files.to_vec();
            fastrand::shuffle(&mut cycle);
            runtime.queue = cycle;
        }

        let position = runtime.queue.iter().position(|path| {
            let key = path_key(path);
            Some(key.as_str()) != current && !used.contains(&key)
        });

        if let Some(index) = position {
            return Some(runtime.queue.remove(index));
        }

        // Only unsuitable images are left in this cycle: start a new one.
        runtime.queue.clear();
    }

    pick_random(files, current, used)
}

/// Any image except the one on this monitor and the ones given to other
/// monitors, relaxing those rules when the folder is too small.
fn pick_random(files: &[PathBuf], current: Option<&str>, used: &[String]) -> Option<PathBuf> {
    let keyed: Vec<(&PathBuf, String)> = files.iter().map(|path| (path, path_key(path))).collect();

    let fresh: Vec<&PathBuf> = keyed
        .iter()
        .filter(|(_, key)| Some(key.as_str()) != current && !used.contains(key))
        .map(|(path, _)| *path)
        .collect();
    if !fresh.is_empty() {
        return fastrand::choice(fresh).cloned();
    }

    let not_current: Vec<&PathBuf> = keyed
        .iter()
        .filter(|(_, key)| Some(key.as_str()) != current)
        .map(|(path, _)| *path)
        .collect();
    if !not_current.is_empty() {
        return fastrand::choice(not_current).cloned();
    }

    fastrand::choice(files).cloned()
}

/// Comparable identity for a path (resolved, case-insensitive on Windows).
fn path_key(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(|name| PathBuf::from(format!("C:/walls/{name}"))).collect()
    }

    #[test]
    fn shuffle_shows_every_image_before_repeating() {
        let files = files(&["a.jpg", "b.jpg", "c.jpg", "d.jpg", "e.jpg"]);
        let mut runtime = RotationRuntime::default();
        let mut current: Option<String> = None;

        for _ in 0..20 {
            let mut seen = Vec::new();
            for _ in 0..files.len() {
                let pick = pick_shuffled(&mut runtime, &files, current.as_deref(), &[]).unwrap();
                let key = path_key(&pick);
                assert_ne!(Some(key.clone()), current, "repeated the image on screen");
                assert!(!seen.contains(&key), "repeated an image within one cycle");
                seen.push(key.clone());
                current = Some(key);
            }
        }
    }

    #[test]
    fn monitors_get_different_images() {
        let files = files(&["a.jpg", "b.jpg", "c.jpg"]);
        let mut runtime = RotationRuntime::default();

        for _ in 0..50 {
            let mut used = Vec::new();
            for _ in 0..3 {
                let pick = pick_shuffled(&mut runtime, &files, None, &used).unwrap();
                used.push(path_key(&pick));
            }
            used.sort();
            used.dedup();
            assert_eq!(used.len(), 3);

            let mut used = Vec::new();
            for _ in 0..3 {
                let pick = pick_random(&files, None, &used).unwrap();
                used.push(path_key(&pick));
            }
            used.sort();
            used.dedup();
            assert_eq!(used.len(), 3);
        }
    }

    #[test]
    fn random_never_repeats_current_when_possible() {
        let files = files(&["a.jpg", "b.jpg"]);
        let current = path_key(&files[0]);
        for _ in 0..50 {
            let pick = pick_random(&files, Some(&current), &[]).unwrap();
            assert_ne!(path_key(&pick), current);
        }
    }

    #[test]
    fn time_asleep_counts_toward_the_interval() {
        let hour = Duration::from_secs(3600);
        let interval = 3 * hour;

        // 1 h awake: not due yet.
        let (elapsed, due) = advance_timer(Duration::ZERO, hour, interval);
        assert!(!due);
        assert_eq!(elapsed, hour);

        // Then the PC sleeps 5 h: the first tick after waking sees a 5 h gap.
        let (elapsed, due) = advance_timer(elapsed, 5 * hour, interval);
        assert!(due, "should change right after waking");
        assert_eq!(elapsed, Duration::ZERO, "count restarts after the change");

        // A short nap that does not reach the interval keeps the progress.
        let (elapsed, due) = advance_timer(hour, hour, interval);
        assert!(!due);
        assert_eq!(elapsed, 2 * hour);
    }

    #[test]
    fn single_image_folder_still_returns_an_image() {
        let files = files(&["only.jpg"]);
        let current = path_key(&files[0]);
        let mut runtime = RotationRuntime::default();
        assert!(pick_shuffled(&mut runtime, &files, Some(&current), &[]).is_some());
        assert!(pick_random(&files, Some(&current), &[]).is_some());
    }
}
