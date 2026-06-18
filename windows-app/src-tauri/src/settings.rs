use crate::models::AppSettings;
use crate::AppState;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("Could not locate app data directory: {err}"))?;
    fs::create_dir_all(&dir).map_err(|err| format!("Could not create settings directory: {err}"))?;
    Ok(dir.join("settings.json"))
}

pub fn read_settings_from_disk(app: &AppHandle) -> AppSettings {
    let Ok(path) = settings_path(app) else {
        return AppSettings::default();
    };
    let Ok(data) = fs::read_to_string(path) else {
        return AppSettings::default();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn write_settings_to_disk(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let data = serde_json::to_string_pretty(settings)
        .map_err(|err| format!("Could not serialize settings: {err}"))?;
    fs::write(path, data).map_err(|err| format!("Could not write settings: {err}"))
}

#[tauri::command]
pub fn load_settings(
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    let settings = read_settings_from_disk(&app);
    *app_state
        .settings
        .lock()
        .map_err(|_| "Settings lock failed".to_string())? = settings.clone();
    Ok(settings)
}

#[tauri::command]
pub fn save_settings(
    settings: AppSettings,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    write_settings_to_disk(&app, &settings)?;
    *app_state
        .settings
        .lock()
        .map_err(|_| "Settings lock failed".to_string())? = settings.clone();
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_mac_ports() {
        let settings = AppSettings::default();
        assert_eq!(settings.browser_server_port, 7373);
        assert_eq!(settings.director_server_port, 7575);
        assert_eq!(settings.notch_width, 340.0);
    }
}

