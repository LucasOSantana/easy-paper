use crate::i18n::{self, Language, SUPPORTED_SETTINGS};
use crate::models::AppConfig;
use crate::services::config_service::modify_config;

/// The language the UI must use, with "system" already resolved.
#[tauri::command]
pub fn get_language() -> Language {
    i18n::current()
}

/// Saves the language setting: "system", "en" or "pt-BR".
#[tauri::command]
pub fn set_language(app: tauri::AppHandle, language: String) -> Result<AppConfig, String> {
    if !SUPPORTED_SETTINGS.contains(&language.as_str()) {
        return Err(crate::tr!(
            en: format!("Unsupported language: {}", language),
            pt: format!("Idioma não suportado: {}", language),
        ));
    }
    modify_config(&app, |config| {
        config.language = language;
        Ok(())
    })
}
