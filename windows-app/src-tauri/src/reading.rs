use crate::models::{
    AppSettings, BrowserState, DirectorState, ListeningMode, ReadingState, StartReadingRequest,
};
use crate::text::{
    build_word_items, char_offset_for_word_progress, split_text_into_words, total_char_count,
    word_progress_for_char_offset,
};
use crate::AppState;
use tauri::State;

pub fn rebuild_reading_content(state: &mut ReadingState) {
    let current_text = state
        .pages
        .get(state.current_page_index)
        .map(|text| text.trim())
        .unwrap_or_default();
    state.words = split_text_into_words(current_text);
    state.word_items = build_word_items(&state.words);
    state.total_char_count = total_char_count(&state.words);
    state.has_next_page = has_next_page(&state.pages, state.current_page_index);
    state.recognized_char_count = state.recognized_char_count.min(state.total_char_count);
}

pub fn has_next_page(pages: &[String], current_page_index: usize) -> bool {
    pages
        .iter()
        .skip(current_page_index + 1)
        .any(|page| !page.trim().is_empty())
}

pub fn next_non_empty_page(pages: &[String], current_page_index: usize) -> Option<usize> {
    pages
        .iter()
        .enumerate()
        .skip(current_page_index + 1)
        .find_map(|(index, page)| (!page.trim().is_empty()).then_some(index))
}

#[tauri::command]
pub fn get_reading_state(app_state: State<'_, AppState>) -> Result<ReadingState, String> {
    app_state
        .reading
        .lock()
        .map(|state| state.clone())
        .map_err(|_| "Reading state lock failed".to_string())
}

#[tauri::command]
pub fn set_pages(
    request: StartReadingRequest,
    app_state: State<'_, AppState>,
) -> Result<ReadingState, String> {
    let settings = app_state
        .settings
        .lock()
        .map_err(|_| "Settings lock failed".to_string())?
        .clone();
    let mut state = app_state
        .reading
        .lock()
        .map_err(|_| "Reading state lock failed".to_string())?;

    let pages = if request.pages.is_empty() {
        vec![String::new()]
    } else {
        request.pages
    };
    state.pages = pages;
    state.current_page_index = request.current_page_index.min(state.pages.len().saturating_sub(1));
    state.read_pages.clear();
    state.is_running = false;
    state.recognized_char_count = 0;
    state.timer_word_progress = 0.0;
    state.listening_mode = settings.listening_mode;
    rebuild_reading_content(&mut state);
    Ok(state.clone())
}

#[tauri::command]
pub fn start_reading(
    request: StartReadingRequest,
    app_state: State<'_, AppState>,
) -> Result<ReadingState, String> {
    let settings = app_state
        .settings
        .lock()
        .map_err(|_| "Settings lock failed".to_string())?
        .clone();
    let mut state = app_state
        .reading
        .lock()
        .map_err(|_| "Reading state lock failed".to_string())?;

    let pages = if request.pages.is_empty() {
        vec![String::new()]
    } else {
        request.pages
    };
    let index = request.current_page_index.min(pages.len().saturating_sub(1));
    let current_has_text = pages.get(index).map(|page| !page.trim().is_empty()).unwrap_or(false);
    if !current_has_text {
        return Err("Current page is empty".to_string());
    }

    state.pages = pages;
    state.current_page_index = index;
    if !state.read_pages.contains(&index) {
        state.read_pages.push(index);
    }
    state.is_running = true;
    state.recognized_char_count = 0;
    state.timer_word_progress = 0.0;
    state.last_spoken_text.clear();
    state.audio_levels = vec![0.0; 30];
    state.listening_mode = settings.listening_mode;
    rebuild_reading_content(&mut state);
    Ok(state.clone())
}

#[tauri::command]
pub fn stop_reading(app_state: State<'_, AppState>) -> Result<ReadingState, String> {
    let mut state = app_state
        .reading
        .lock()
        .map_err(|_| "Reading state lock failed".to_string())?;
    state.is_running = false;
    state.recognized_char_count = 0;
    state.timer_word_progress = 0.0;
    Ok(state.clone())
}

#[tauri::command]
pub fn next_page(app_state: State<'_, AppState>) -> Result<ReadingState, String> {
    let mut state = app_state
        .reading
        .lock()
        .map_err(|_| "Reading state lock failed".to_string())?;
    let Some(next_index) = next_non_empty_page(&state.pages, state.current_page_index) else {
        return Err("No next page".to_string());
    };

    state.current_page_index = next_index;
    if !state.read_pages.contains(&next_index) {
        state.read_pages.push(next_index);
    }
    state.recognized_char_count = 0;
    state.timer_word_progress = 0.0;
    rebuild_reading_content(&mut state);
    Ok(state.clone())
}

#[tauri::command]
pub fn jump_to_char(char_offset: usize, app_state: State<'_, AppState>) -> Result<ReadingState, String> {
    let mut state = app_state
        .reading
        .lock()
        .map_err(|_| "Reading state lock failed".to_string())?;
    state.recognized_char_count = char_offset.min(state.total_char_count);
    state.timer_word_progress = word_progress_for_char_offset(&state.words, state.recognized_char_count);
    Ok(state.clone())
}

#[tauri::command]
pub fn update_timer_progress(
    word_progress: f64,
    app_state: State<'_, AppState>,
) -> Result<ReadingState, String> {
    let mut state = app_state
        .reading
        .lock()
        .map_err(|_| "Reading state lock failed".to_string())?;
    state.timer_word_progress = word_progress.max(0.0);
    state.recognized_char_count =
        char_offset_for_word_progress(&state.words, state.timer_word_progress);
    Ok(state.clone())
}

#[tauri::command]
pub fn update_director_text(
    text: String,
    read_char_count: usize,
    app_state: State<'_, AppState>,
) -> Result<ReadingState, String> {
    let mut state = app_state
        .reading
        .lock()
        .map_err(|_| "Reading state lock failed".to_string())?;
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        return Err("Director text is empty".to_string());
    }
    state.pages = vec![trimmed];
    state.current_page_index = 0;
    state.is_running = true;
    state.recognized_char_count = read_char_count;
    rebuild_reading_content(&mut state);
    Ok(state.clone())
}

pub fn browser_state(reading: &ReadingState, settings: &AppSettings) -> BrowserState {
    let effective = reading.recognized_char_count.min(reading.total_char_count);
    BrowserState {
        words: reading.words.clone(),
        highlighted_char_count: effective,
        total_char_count: reading.total_char_count,
        audio_levels: reading.audio_levels.clone(),
        is_listening: reading.is_running && reading.listening_mode != ListeningMode::Classic,
        is_done: reading.total_char_count > 0 && effective >= reading.total_char_count,
        font_color: settings.font_color_preset.css_color().to_string(),
        cue_color: settings.cue_color_preset.css_color().to_string(),
        has_next_page: reading.has_next_page,
        is_active: reading.is_running,
        highlight_words: reading.listening_mode == ListeningMode::WordTracking,
        last_spoken_text: reading.last_spoken_text.clone(),
    }
}

pub fn director_state(reading: &ReadingState, settings: &AppSettings) -> DirectorState {
    let effective = reading.recognized_char_count.min(reading.total_char_count);
    DirectorState {
        words: reading.words.clone(),
        highlighted_char_count: effective,
        total_char_count: reading.total_char_count,
        is_active: reading.is_running,
        is_done: reading.total_char_count > 0 && effective >= reading.total_char_count,
        is_listening: reading.is_running && reading.listening_mode != ListeningMode::Classic,
        font_color: settings.font_color_preset.css_color().to_string(),
        cue_color: settings.cue_color_preset.css_color().to_string(),
        last_spoken_text: reading.last_spoken_text.clone(),
        audio_levels: reading.audio_levels.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ReadingState;

    #[test]
    fn finds_next_non_empty_page() {
        let pages = vec!["one".into(), "  ".into(), "two".into()];
        assert_eq!(next_non_empty_page(&pages, 0), Some(2));
        assert_eq!(next_non_empty_page(&pages, 2), None);
    }

    #[test]
    fn rebuilds_current_content() {
        let mut state = ReadingState {
            pages: vec!["Hello 世界 [pause]".into()],
            is_running: true,
            ..ReadingState::default()
        };
        rebuild_reading_content(&mut state);
        assert_eq!(state.words, vec!["Hello", "世", "界", "[pause]"]);
        assert_eq!(state.total_char_count, "Hello 世 界 [pause]".chars().count());
        assert!(state.word_items[3].is_annotation);
    }
}
