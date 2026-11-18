use crate::models::{
    AppConfig, AutoRotateConfig, MAX_ROTATION_INTERVAL_MINUTES, MIN_ROTATION_INTERVAL_MINUTES,
};
use crate::services::config_service::modify_config;
use crate::services::rotation_service::reset_rotation_timer;

#[tauri::command]
pub fn set_auto_rotate(
    app: tauri::AppHandle,
    settings: AutoRotateConfig,
) -> Result<AppConfig, String> {
    if !(MIN_ROTATION_INTERVAL_MINUTES..=MAX_ROTATION_INTERVAL_MINUTES)
        .contains(&settings.interval_minutes)
    {
        let days = MAX_ROTATION_INTERVAL_MINUTES / (24 * 60);
        return Err(crate::tr!(
            en: format!("The interval must be between {} minutes and {} days.", MIN_ROTATION_INTERVAL_MINUTES, days),
            pt: format!("O intervalo deve ficar entre {} minutos e {} dias.", MIN_ROTATION_INTERVAL_MINUTES, days),
        ));
    }

    let updated_config = modify_config(&app, |config| {
        config.auto_rotate = settings;
        Ok(())
    })?;

    reset_rotation_timer(&app);
    Ok(updated_config)
}
