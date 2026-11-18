//! Actions on single wallpapers of the library: favorites, deleting (to the
//! Recycle Bin) and showing a file in Explorer.

use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Runtime};

use crate::services::config_service::{get_images_directory, modify_config};
use crate::services::thumbnail_service::thumbnail_path_for;

/// Resolves `path` and checks that it is an image directly inside the
/// configured wallpaper folder, so the UI can never touch other files.
pub fn resolve_library_file<R: Runtime>(app: &AppHandle<R>, path: &str) -> Result<PathBuf, String> {
    let folder = get_images_directory(app);
    let folder = dunce::canonicalize(&folder)
        .map_err(|_| crate::tr!(
            en: "The wallpaper folder is not available.",
            pt: "A pasta de papéis de parede não está disponível.",
        ))?;
    let file = dunce::canonicalize(path)
        .map_err(|_| crate::tr!(en: "This wallpaper no longer exists.", pt: "Este papel de parede não existe mais."))?;

    if !file.is_file() || file.parent() != Some(folder.as_path()) {
        return Err(crate::tr!(
            en: "This file is not in your wallpaper folder.",
            pt: "Este arquivo não está na sua pasta de papéis de parede.",
        ));
    }
    Ok(file)
}

fn file_name_of(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .ok_or_else(|| crate::tr!(en: "Invalid file name.", pt: "Nome de arquivo inválido."))
}

/// Marks or unmarks a wallpaper as favorite. Returns the new state.
pub fn set_favorite<R: Runtime>(app: &AppHandle<R>, path: &str, favorite: bool) -> Result<bool, String> {
    let file = resolve_library_file(app, path)?;
    let name = file_name_of(&file)?;

    modify_config(app, |config| {
        config.favorites.retain(|existing| !existing.eq_ignore_ascii_case(&name));
        if favorite {
            config.favorites.push(name.clone());
            config.favorites.sort_by_key(|n| n.to_lowercase());
        }
        Ok(())
    })?;

    Ok(favorite)
}

/// Moves a wallpaper to the Recycle Bin (so it can be restored), removes its
/// cached thumbnail and drops it from the favorites.
pub fn delete_wallpaper<R: Runtime>(app: &AppHandle<R>, path: &str) -> Result<(), String> {
    let file = resolve_library_file(app, path)?;
    let name = file_name_of(&file)?;

    move_to_recycle_bin(&file)?;

    if let Some(folder) = file.parent() {
        let _ = fs::remove_file(thumbnail_path_for(&file, folder));
    }

    modify_config(app, |config| {
        config.favorites.retain(|existing| !existing.eq_ignore_ascii_case(&name));
        Ok(())
    })?;
    Ok(())
}

/// Opens Explorer with the wallpaper selected.
pub fn reveal_in_explorer<R: Runtime>(app: &AppHandle<R>, path: &str) -> Result<(), String> {
    let file = resolve_library_file(app, path)?;

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("explorer.exe")
            .raw_arg(format!("/select,\"{}\"", file.display()))
            .spawn()
            .map_err(|e| crate::tr!(
                en: format!("Could not open Explorer: {}", e),
                pt: format!("Não foi possível abrir o Explorer: {}", e),
            ))?;
    }
    #[cfg(not(windows))]
    let _ = file;

    Ok(())
}

#[cfg(windows)]
fn move_to_recycle_bin(file: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{
        SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT,
        FO_DELETE, SHFILEOPSTRUCTW,
    };

    // pFrom is a list of paths ended by an extra NUL.
    let mut from: Vec<u16> = file.as_os_str().encode_wide().collect();
    from.push(0);
    from.push(0);

    let mut operation = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: PCWSTR(from.as_ptr()),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT).0 as u16,
        ..Default::default()
    };

    let result = unsafe { SHFileOperationW(&mut operation) };
    if result != 0 || operation.fAnyOperationsAborted.as_bool() {
        return Err(crate::tr!(
            en: format!("Could not move the wallpaper to the Recycle Bin (error {}).", result),
            pt: format!("Não foi possível mover o papel de parede para a Lixeira (erro {}).", result),
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
fn move_to_recycle_bin(file: &Path) -> Result<(), String> {
    fs::remove_file(file).map_err(|e| e.to_string())
}
