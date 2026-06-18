use crate::models::OverlayMode;
use crate::AppState;
use serde::Serialize;
use tauri::{AppHandle, LogicalPosition, Manager, State, WebviewUrl, WebviewWindowBuilder};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub id: u32,
    pub name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

#[tauri::command]
pub fn list_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let monitors = app
        .available_monitors()
        .map_err(|err| format!("Could not list monitors: {err}"))?;
    Ok(monitors
        .into_iter()
        .enumerate()
        .map(|(id, monitor)| {
            let pos = monitor.position();
            let size = monitor.size();
            MonitorInfo {
                id: id as u32,
                name: monitor.name().map(ToString::to_string),
                x: pos.x,
                y: pos.y,
                width: size.width,
                height: size.height,
                scale_factor: monitor.scale_factor(),
            }
        })
        .collect())
}

#[tauri::command]
pub fn show_overlay_window(
    mode: OverlayMode,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let settings = app_state
        .settings
        .lock()
        .map_err(|_| "Settings lock failed".to_string())?
        .clone();
    let label = match mode {
        OverlayMode::Pinned => "textream-pinned-overlay",
        OverlayMode::Floating => "textream-floating-overlay",
        OverlayMode::Fullscreen => "textream-fullscreen-overlay",
    };

    if let Some(existing) = app.get_webview_window(label) {
        let _ = existing.close();
    }

    let route = match mode {
        OverlayMode::Pinned => "overlay-pinned",
        OverlayMode::Floating => "overlay-floating",
        OverlayMode::Fullscreen => "overlay-fullscreen",
    };
    let width = settings.notch_width.max(310.0);
    let height = settings.text_area_height.max(100.0) + 88.0;
    let builder = WebviewWindowBuilder::new(
        &app,
        label,
        WebviewUrl::App(format!("index.html#/{route}").into()),
    )
    .title("Textream")
    .decorations(false)
    .always_on_top(true)
    .resizable(mode == OverlayMode::Floating)
    .inner_size(width, height);

    let window = builder
        .build()
        .map_err(|err| format!("Could not create overlay window: {err}"))?;

    match mode {
        OverlayMode::Pinned => {
            if let Ok(Some(monitor)) = window.current_monitor() {
                let pos = monitor.position();
                let size = monitor.size();
                let x = pos.x as f64 + (size.width as f64 - width) / 2.0;
                let _ = window.set_position(LogicalPosition::new(x, pos.y as f64));
            }
        }
        OverlayMode::Floating => {
            let _ = window.set_position(LogicalPosition::new(120.0, 120.0));
        }
        OverlayMode::Fullscreen => {
            let _ = window.set_fullscreen(true);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn close_overlay_windows(app: AppHandle) -> Result<(), String> {
    for label in [
        "textream-pinned-overlay",
        "textream-floating-overlay",
        "textream-fullscreen-overlay",
    ] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.close();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_capture_protection(_enabled: bool) -> Result<bool, String> {
    Ok(cfg!(target_os = "windows"))
}
