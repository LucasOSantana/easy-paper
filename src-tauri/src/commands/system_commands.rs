#[tauri::command]
pub async fn pick_folder() -> Option<String> {
    let result = rfd::AsyncFileDialog::new()
        .set_title(crate::tr!(en: "Select Wallpaper Folder", pt: "Selecione a pasta de papéis de parede"))
        .pick_folder()
        .await;

    result.map(|handle| handle.path().to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_folder_path(path: String) {
    if !path.is_empty() {
        let _ = std::process::Command::new("explorer.exe")
            .arg(&path)
            .spawn();
    }
}
