use crate::models::{AudioInputDevice, SpeechBackendInfo, SpeechBackendKind, SpeechState};
use crate::reading::rebuild_reading_content;
use crate::text::normalize_for_matching;
use crate::AppState;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone)]
pub struct SpeechManager {
    pub backend: SpeechBackendKind,
    pub is_listening: bool,
    pub error: Option<String>,
}

impl Default for SpeechManager {
    fn default() -> Self {
        Self {
            backend: SpeechBackendKind::WindowsNative,
            is_listening: false,
            error: None,
        }
    }
}

#[tauri::command]
pub fn list_speech_backends() -> Vec<SpeechBackendInfo> {
    vec![
        SpeechBackendInfo {
            id: SpeechBackendKind::WindowsNative,
            label: "Windows Native".to_string(),
            available: cfg!(target_os = "windows"),
            detail: if cfg!(target_os = "windows") {
                "Uses the Windows speech recognition stack.".to_string()
            } else {
                "Available when this app is built and run on Windows.".to_string()
            },
        },
        SpeechBackendInfo {
            id: SpeechBackendKind::WebSpeech,
            label: "Web Speech".to_string(),
            available: false,
            detail: "Reserved fallback; WebView2 support is not guaranteed.".to_string(),
        },
        SpeechBackendInfo {
            id: SpeechBackendKind::LocalModel,
            label: "Local Model".to_string(),
            available: false,
            detail: "Reserved for a packaged Whisper/Vosk style backend.".to_string(),
        },
    ]
}

#[tauri::command]
pub fn set_speech_backend(
    backend: SpeechBackendKind,
    app_state: State<'_, AppState>,
) -> Result<SpeechState, String> {
    let mut speech = app_state
        .speech
        .lock()
        .map_err(|_| "Speech state lock failed".to_string())?;
    speech.backend = backend;
    speech.error = None;
    Ok(speech_state_from_parts(&speech, &app_state)?)
}

#[tauri::command]
pub fn start_speech(
    text: String,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<SpeechState, String> {
    let mut speech = app_state
        .speech
        .lock()
        .map_err(|_| "Speech state lock failed".to_string())?;

    if speech.backend == SpeechBackendKind::WindowsNative && !cfg!(target_os = "windows") {
        speech.error = Some("Windows Native speech is only available on Windows builds.".to_string());
        speech.is_listening = false;
    } else {
        speech.error = None;
        speech.is_listening = true;
    }

    {
        let mut reading = app_state
            .reading
            .lock()
            .map_err(|_| "Reading state lock failed".to_string())?;
        if !text.trim().is_empty() {
            reading.pages = vec![text];
            reading.current_page_index = 0;
            reading.recognized_char_count = 0;
            reading.last_spoken_text.clear();
            reading.audio_levels = vec![0.0; 30];
            rebuild_reading_content(&mut reading);
        }
    }

    let state = speech_state_from_parts(&speech, &app_state)?;
    let _ = app.emit("speech-state", &state);
    Ok(state)
}

#[tauri::command]
pub fn stop_speech(app: AppHandle, app_state: State<'_, AppState>) -> Result<SpeechState, String> {
    let mut speech = app_state
        .speech
        .lock()
        .map_err(|_| "Speech state lock failed".to_string())?;
    speech.is_listening = false;
    let state = speech_state_from_parts(&speech, &app_state)?;
    let _ = app.emit("speech-state", &state);
    Ok(state)
}

#[tauri::command]
pub fn apply_spoken_text(
    spoken_text: String,
    app: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<SpeechState, String> {
    let normalized = normalize_for_matching(&spoken_text);
    let mut reading = app_state
        .reading
        .lock()
        .map_err(|_| "Reading state lock failed".to_string())?;
    reading.last_spoken_text = spoken_text;
    let collapsed = reading.words.join(" ");
    let normalized_source = normalize_for_matching(&collapsed);
    if let Some(pos) = normalized_source.find(&normalized) {
        let next = pos + normalized.len();
        reading.recognized_char_count = next.min(reading.total_char_count);
    }
    drop(reading);

    let speech = app_state
        .speech
        .lock()
        .map_err(|_| "Speech state lock failed".to_string())?;
    let state = speech_state_from_parts(&speech, &app_state)?;
    let _ = app.emit("speech-state", &state);
    Ok(state)
}

#[tauri::command]
pub fn list_audio_inputs() -> Vec<AudioInputDevice> {
    vec![AudioInputDevice {
        id: "default".to_string(),
        name: if cfg!(target_os = "windows") {
            "Default Windows microphone".to_string()
        } else {
            "Default microphone (Windows enumeration available on Windows builds)".to_string()
        },
    }]
}

fn speech_state_from_parts(
    speech: &SpeechManager,
    app_state: &State<'_, AppState>,
) -> Result<SpeechState, String> {
    let reading = app_state
        .reading
        .lock()
        .map_err(|_| "Reading state lock failed".to_string())?;
    Ok(SpeechState {
        backend: speech.backend.clone(),
        recognized_char_count: reading.recognized_char_count,
        audio_levels: reading.audio_levels.clone(),
        last_spoken_text: reading.last_spoken_text.clone(),
        is_listening: speech.is_listening,
        error: speech.error.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_windows_native_as_default_backend() {
        let manager = SpeechManager::default();
        assert_eq!(manager.backend, SpeechBackendKind::WindowsNative);
    }
}

