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
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    Emitter, Manager,
};

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

            let handle = app.handle();
            let settings_item = MenuItemBuilder::with_id("settings", "Settings...")
                .accelerator("CmdOrCtrl+Comma")
                .build(handle)?;
            let update_item = MenuItemBuilder::with_id("check-updates", "Check for Updates...")
                .build(handle)?;
            let about_item = MenuItemBuilder::with_id("about", "About Textream").build(handle)?;
            let open_item = MenuItemBuilder::with_id("open", "Open File or Presentation...")
                .accelerator("CmdOrCtrl+O")
                .build(handle)?;
            let save_item = MenuItemBuilder::with_id("save", "Save")
                .accelerator("CmdOrCtrl+S")
                .build(handle)?;
            let save_as_item = MenuItemBuilder::with_id("save-as", "Save As...")
                .accelerator("CmdOrCtrl+Shift+S")
                .build(handle)?;
            let add_page_item = MenuItemBuilder::with_id("add-page", "Add Page")
                .accelerator("CmdOrCtrl+N")
                .build(handle)?;
            let help_item = MenuItemBuilder::with_id("help", "Textream Help").build(handle)?;

            let app_menu = SubmenuBuilder::new(handle, "Textream")
                .item(&about_item)
                .separator()
                .item(&update_item)
                .item(&settings_item)
                .separator()
                .quit()
                .build()?;
            let file_menu = SubmenuBuilder::new(handle, "File")
                .item(&open_item)
                .item(&save_item)
                .item(&save_as_item)
                .separator()
                .item(&add_page_item)
                .separator()
                .close_window()
                .build()?;
            let edit_menu = SubmenuBuilder::new(handle, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;
            let view_menu = SubmenuBuilder::new(handle, "View").fullscreen().build()?;
            let window_menu = SubmenuBuilder::new(handle, "Window")
                .minimize()
                .maximize()
                .build()?;
            let help_menu = SubmenuBuilder::new(handle, "Help").item(&help_item).build()?;
            let menu = MenuBuilder::new(handle)
                .items(&[&app_menu, &file_menu, &edit_menu, &view_menu, &window_menu, &help_menu])
                .build()?;
            app.set_menu(menu)?;
            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" | "check-updates" | "open" | "save" | "save-as" | "add-page" => {
                let _ = app.emit("menu-action", event.id().as_ref());
            }
            "about" => {
                let _ = app.emit("menu-action", "settings");
            }
            "help" => {
                let _ = tauri_plugin_opener::OpenerExt::opener(app)
                    .open_url("https://github.com/yourpapayouknow/TextreamWindows", None::<&str>);
            }
            _ => {}
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
