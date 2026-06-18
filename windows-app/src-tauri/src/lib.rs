mod director_server;
mod documents;
mod models;
mod pptx;
mod reading;
mod remote_server;
mod settings;
mod speech;
mod text;
mod updates;
mod windows_overlay;

use models::{AppSettings, ReadingState};
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub reading: Arc<Mutex<ReadingState>>,
    pub speech: Arc<Mutex<speech::SpeechManager>>,
    pub remote: Mutex<remote_server::RemoteServer>,
    pub director: Mutex<director_server::DirectorServer>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Arc::new(Mutex::new(AppSettings::default())),
            reading: Arc::new(Mutex::new(ReadingState::default())),
            speech: Arc::new(Mutex::new(speech::SpeechManager::default())),
            remote: Mutex::new(remote_server::RemoteServer::default()),
            director: Mutex::new(director_server::DirectorServer::default()),
        }
    }
}

#[tauri::command]
fn get_launch_urls() -> Vec<String> {
    std::env::args()
        .filter(|arg| arg.starts_with("textream://"))
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .setup(|app| {
            let settings = settings::read_settings_from_disk(app.handle());
            let state = app.state::<AppState>();
            if let Ok(mut target) = state.settings.lock() {
                *target = settings;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_launch_urls,
            documents::open_document,
            documents::save_document,
            documents::extract_pptx_notes,
            settings::load_settings,
            settings::save_settings,
            reading::get_reading_state,
            reading::set_pages,
            reading::start_reading,
            reading::stop_reading,
            reading::next_page,
            reading::jump_to_char,
            reading::update_timer_progress,
            reading::update_director_text,
            speech::list_speech_backends,
            speech::set_speech_backend,
            speech::start_speech,
            speech::stop_speech,
            speech::apply_spoken_text,
            speech::list_audio_inputs,
            remote_server::start_remote_server,
            remote_server::stop_remote_server,
            director_server::start_director_server,
            director_server::stop_director_server,
            windows_overlay::list_monitors,
            windows_overlay::show_overlay_window,
            windows_overlay::close_overlay_windows,
            windows_overlay::set_capture_protection,
            updates::check_for_updates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
