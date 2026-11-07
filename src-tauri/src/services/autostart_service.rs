pub fn enable_windows_autostart() {
    #[cfg(target_os = "windows")]
    {
        if let Ok(exe_path) = std::env::current_exe() {
            let exe_str = exe_path.to_string_lossy().to_string();
            let _ = std::process::Command::new("reg")
                .args([
                    "add",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    "easy-paper",
                    "/t",
                    "REG_SZ",
                    "/d",
                    &format!(
                        "\"{}\" {}",
                        exe_str,
                        crate::services::rotation_service::AUTOSTART_ARG
                    ),
                    "/f",
                ])
                .spawn();
        }
    }
}
