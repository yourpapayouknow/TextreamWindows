export type FontSizePreset = "xs" | "sm" | "lg" | "xl";
export type FontFamilyPreset = "sans" | "serif" | "mono" | "dyslexia";
export type FontColorPreset = "white" | "yellow" | "green" | "blue" | "pink" | "orange";
export type CueBrightness = "dim" | "low" | "medium" | "bright";
export type OverlayMode = "pinned" | "floating" | "fullscreen";
export type ExternalDisplayMode = "off" | "teleprompter" | "mirror";
export type MirrorAxis = "horizontal" | "vertical" | "both";
export type ListeningMode = "wordTracking" | "classic" | "silencePaused";
export type SpeechBackendKind = "windowsNative" | "webSpeech" | "localModel";

export interface AppSettings {
  notchWidth: number;
  textAreaHeight: number;
  speechLocale: string;
  fontSizePreset: FontSizePreset;
  fontFamilyPreset: FontFamilyPreset;
  fontColorPreset: FontColorPreset;
  cueColorPreset: FontColorPreset;
  cueBrightness: CueBrightness;
  overlayMode: OverlayMode;
  notchDisplayMode: "followMouse" | "fixedDisplay";
  pinnedScreenId: number;
  floatingGlassEffect: boolean;
  glassOpacity: number;
  overlayTransparency: boolean;
  overlayTransparencyOpacity: number;
  followCursorWhenUndocked: boolean;
  externalDisplayMode: ExternalDisplayMode;
  externalScreenId: number;
  mirrorAxis: MirrorAxis;
  listeningMode: ListeningMode;
  scrollSpeed: number;
  hideFromScreenShare: boolean;
  showElapsedTime: boolean;
  selectedMicUid: string;
  autoNextPage: boolean;
  autoNextPageDelay: number;
  fullscreenScreenId: number;
  browserServerEnabled: boolean;
  browserServerPort: number;
  directorModeEnabled: boolean;
  directorServerPort: number;
  speechBackend: SpeechBackendKind;
}

export interface WordItem {
  id: number;
  word: string;
  charOffset: number;
  isAnnotation: boolean;
}

export interface ReadingState {
  pages: string[];
  currentPageIndex: number;
  readPages: number[];
  isRunning: boolean;
  words: string[];
  wordItems: WordItem[];
  totalCharCount: number;
  recognizedCharCount: number;
  timerWordProgress: number;
  hasNextPage: boolean;
  listeningMode: ListeningMode;
  lastSpokenText: string;
  audioLevels: number[];
}

export interface SpeechBackendInfo {
  id: SpeechBackendKind;
  label: string;
  available: boolean;
  detail: string;
}

export interface SpeechState {
  backend: SpeechBackendKind;
  recognizedCharCount: number;
  audioLevels: number[];
  lastSpokenText: string;
  isListening: boolean;
  error?: string | null;
}

export interface MonitorInfo {
  id: number;
  name?: string | null;
  x: number;
  y: number;
  width: number;
  height: number;
  scaleFactor: number;
}

export interface UpdateStatus {
  currentVersion: string;
  latestVersion?: string | null;
  releaseUrl?: string | null;
  isUpdateAvailable: boolean;
  error?: string | null;
}

export const colorMap: Record<FontColorPreset, string> = {
  white: "#ffffff",
  yellow: "rgb(255,214,10)",
  green: "rgb(51,214,74)",
  blue: "rgb(79,140,255)",
  pink: "rgb(255,97,145)",
  orange: "rgb(255,158,10)",
};

export const fontSizeMap: Record<FontSizePreset, number> = {
  xs: 14,
  sm: 16,
  lg: 20,
  xl: 24,
};

