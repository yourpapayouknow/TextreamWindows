use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FontSizePreset {
    Xs,
    Sm,
    Lg,
    Xl,
}

impl FontSizePreset {
    pub fn point_size(&self) -> u16 {
        match self {
            Self::Xs => 14,
            Self::Sm => 16,
            Self::Lg => 20,
            Self::Xl => 24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FontFamilyPreset {
    Sans,
    Serif,
    Mono,
    Dyslexia,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FontColorPreset {
    White,
    Yellow,
    Green,
    Blue,
    Pink,
    Orange,
}

impl FontColorPreset {
    pub fn css_color(&self) -> &'static str {
        match self {
            Self::White => "#ffffff",
            Self::Yellow => "rgb(255,214,10)",
            Self::Green => "rgb(51,214,74)",
            Self::Blue => "rgb(79,140,255)",
            Self::Pink => "rgb(255,97,145)",
            Self::Orange => "rgb(255,158,10)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CueBrightness {
    Dim,
    Low,
    Medium,
    Bright,
}

impl CueBrightness {
    pub fn unread_opacity(&self) -> f64 {
        match self {
            Self::Dim => 0.2,
            Self::Low => 0.35,
            Self::Medium => 0.5,
            Self::Bright => 0.8,
        }
    }

    pub fn read_opacity(&self) -> f64 {
        match self {
            Self::Dim => 0.5,
            Self::Low => 0.6,
            Self::Medium => 0.7,
            Self::Bright => 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OverlayMode {
    Pinned,
    Floating,
    Fullscreen,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NotchDisplayMode {
    FollowMouse,
    FixedDisplay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExternalDisplayMode {
    Off,
    Teleprompter,
    Mirror,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MirrorAxis {
    Horizontal,
    Vertical,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ListeningMode {
    WordTracking,
    Classic,
    SilencePaused,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SpeechBackendKind {
    WindowsNative,
    WebSpeech,
    LocalModel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub notch_width: f64,
    pub text_area_height: f64,
    pub speech_locale: String,
    pub font_size_preset: FontSizePreset,
    pub font_family_preset: FontFamilyPreset,
    pub font_color_preset: FontColorPreset,
    pub cue_color_preset: FontColorPreset,
    pub cue_brightness: CueBrightness,
    pub overlay_mode: OverlayMode,
    pub notch_display_mode: NotchDisplayMode,
    pub pinned_screen_id: u32,
    pub floating_glass_effect: bool,
    pub glass_opacity: f64,
    pub overlay_transparency: bool,
    pub overlay_transparency_opacity: f64,
    pub follow_cursor_when_undocked: bool,
    pub external_display_mode: ExternalDisplayMode,
    pub external_screen_id: u32,
    pub mirror_axis: MirrorAxis,
    pub listening_mode: ListeningMode,
    pub scroll_speed: f64,
    pub hide_from_screen_share: bool,
    pub show_elapsed_time: bool,
    pub selected_mic_uid: String,
    pub auto_next_page: bool,
    pub auto_next_page_delay: u32,
    pub fullscreen_screen_id: u32,
    pub browser_server_enabled: bool,
    pub browser_server_port: u16,
    pub director_mode_enabled: bool,
    pub director_server_port: u16,
    pub speech_backend: SpeechBackendKind,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            notch_width: 340.0,
            text_area_height: 150.0,
            speech_locale: "en-US".to_string(),
            font_size_preset: FontSizePreset::Lg,
            font_family_preset: FontFamilyPreset::Sans,
            font_color_preset: FontColorPreset::White,
            cue_color_preset: FontColorPreset::White,
            cue_brightness: CueBrightness::Dim,
            overlay_mode: OverlayMode::Pinned,
            notch_display_mode: NotchDisplayMode::FollowMouse,
            pinned_screen_id: 0,
            floating_glass_effect: false,
            glass_opacity: 0.15,
            overlay_transparency: false,
            overlay_transparency_opacity: 0.85,
            follow_cursor_when_undocked: false,
            external_display_mode: ExternalDisplayMode::Off,
            external_screen_id: 0,
            mirror_axis: MirrorAxis::Horizontal,
            listening_mode: ListeningMode::WordTracking,
            scroll_speed: 3.0,
            hide_from_screen_share: true,
            show_elapsed_time: true,
            selected_mic_uid: String::new(),
            auto_next_page: false,
            auto_next_page_delay: 3,
            fullscreen_screen_id: 0,
            browser_server_enabled: false,
            browser_server_port: 7373,
            director_mode_enabled: false,
            director_server_port: 7575,
            speech_backend: SpeechBackendKind::WindowsNative,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WordItem {
    pub id: usize,
    pub word: String,
    pub char_offset: usize,
    pub is_annotation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReadingState {
    pub pages: Vec<String>,
    pub current_page_index: usize,
    pub read_pages: Vec<usize>,
    pub is_running: bool,
    pub words: Vec<String>,
    pub word_items: Vec<WordItem>,
    pub total_char_count: usize,
    pub recognized_char_count: usize,
    pub timer_word_progress: f64,
    pub has_next_page: bool,
    pub listening_mode: ListeningMode,
    pub last_spoken_text: String,
    pub audio_levels: Vec<f64>,
}

impl Default for ReadingState {
    fn default() -> Self {
        Self {
            pages: vec![String::new()],
            current_page_index: 0,
            read_pages: Vec::new(),
            is_running: false,
            words: Vec::new(),
            word_items: Vec::new(),
            total_char_count: 0,
            recognized_char_count: 0,
            timer_word_progress: 0.0,
            has_next_page: false,
            listening_mode: ListeningMode::WordTracking,
            last_spoken_text: String::new(),
            audio_levels: vec![0.0; 30],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartReadingRequest {
    pub pages: Vec<String>,
    pub current_page_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDocumentRequest {
    pub path: String,
    pub pages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpeechBackendInfo {
    pub id: SpeechBackendKind,
    pub label: String,
    pub available: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpeechState {
    pub backend: SpeechBackendKind,
    pub recognized_char_count: usize,
    pub audio_levels: Vec<f64>,
    pub last_spoken_text: String,
    pub is_listening: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BrowserState {
    pub words: Vec<String>,
    pub highlighted_char_count: usize,
    pub total_char_count: usize,
    pub audio_levels: Vec<f64>,
    pub is_listening: bool,
    pub is_done: bool,
    pub font_color: String,
    pub cue_color: String,
    pub has_next_page: bool,
    pub is_active: bool,
    pub highlight_words: bool,
    pub last_spoken_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DirectorState {
    pub words: Vec<String>,
    pub highlighted_char_count: usize,
    pub total_char_count: usize,
    pub is_active: bool,
    pub is_done: bool,
    pub is_listening: bool,
    pub font_color: String,
    pub cue_color: String,
    pub last_spoken_text: String,
    pub audio_levels: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DirectorCommand {
    pub r#type: String,
    pub text: Option<String>,
    pub read_char_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AudioInputDevice {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_url: Option<String>,
    pub is_update_available: bool,
    pub error: Option<String>,
}

