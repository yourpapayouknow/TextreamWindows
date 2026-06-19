import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  AudioLines,
  Check,
  CheckCircle2,
  CircleStop,
  Copy,
  ExternalLink,
  Languages,
  Megaphone,
  Mic,
  MonitorUp,
  Palette,
  PanelsTopLeft,
  Pause,
  Play,
  Plus,
  Radio,
  RotateCcw,
  Settings,
  Trash2,
  X,
} from "lucide-react";
import "./App.css";
import {
  AppSettings,
  FontColorPreset,
  ListeningMode,
  MonitorInfo,
  OverlayMode,
  ReadingState,
  SpeechBackendInfo,
  SpeechBackendKind,
  UpdateStatus,
  colorMap,
  fontSizeMap,
} from "./shared/types";
import { buildWordItems, charOffsetForWordProgress, splitTextIntoWords } from "./shared/text";

type SettingsTab = "appearance" | "guidance" | "teleprompter" | "external" | "remote" | "director";

interface AudioInputDevice {
  id: string;
  name: string;
}

const defaultPages = [
  `Welcome to Textream! This is your personal teleprompter that sits right below the top of your screen. [smile]

As you read aloud, the text will highlight in real-time, following your voice. The speech recognition matches your words and keeps track of your progress. [pause]

You can pause at any time, go back and re-read sections, and the highlighting will follow along. When you finish reading all the text, the overlay will automatically close with a smooth animation. [nod]

Try reading this passage out loud to see how the highlighting works. The waveform at the bottom shows your voice activity, and you'll see the last few words you spoke displayed next to it.

Happy presenting! [wave]`,
];

const defaultSettings: AppSettings = {
  notchWidth: 340,
  textAreaHeight: 150,
  speechLocale: "en-US",
  fontSizePreset: "lg",
  fontFamilyPreset: "sans",
  fontColorPreset: "white",
  cueColorPreset: "white",
  cueBrightness: "dim",
  overlayMode: "pinned",
  notchDisplayMode: "followMouse",
  pinnedScreenId: 0,
  floatingGlassEffect: false,
  glassOpacity: 0.15,
  overlayTransparency: false,
  overlayTransparencyOpacity: 0.85,
  followCursorWhenUndocked: false,
  externalDisplayMode: "off",
  externalScreenId: 0,
  mirrorAxis: "horizontal",
  listeningMode: "wordTracking",
  scrollSpeed: 3,
  hideFromScreenShare: true,
  showElapsedTime: true,
  selectedMicUid: "",
  autoNextPage: false,
  autoNextPageDelay: 3,
  fullscreenScreenId: 0,
  browserServerEnabled: false,
  browserServerPort: 7373,
  directorModeEnabled: false,
  directorServerPort: 7575,
  speechBackend: "windowsNative",
};

const settingTabs: Array<{ id: SettingsTab; label: string; icon: ReactNode }> = [
  { id: "appearance", label: "Appearance", icon: <Palette size={20} /> },
  { id: "guidance", label: "Guidance", icon: <AudioLines size={20} /> },
  { id: "teleprompter", label: "Teleprompter", icon: <PanelsTopLeft size={20} /> },
  { id: "external", label: "External", icon: <MonitorUp size={20} /> },
  { id: "remote", label: "Remote", icon: <Radio size={20} /> },
  { id: "director", label: "Director", icon: <Megaphone size={20} /> },
];

function buildLocalReading(pages: string[], currentPageIndex: number, settings: AppSettings): ReadingState {
  const words = splitTextIntoWords(pages[currentPageIndex] ?? "");
  return {
    pages,
    currentPageIndex,
    readPages: [],
    isRunning: false,
    words,
    wordItems: buildWordItems(words),
    totalCharCount: words.join(" ").length,
    recognizedCharCount: 0,
    timerWordProgress: 0,
    hasNextPage: pages.slice(currentPageIndex + 1).some((page) => page.trim()),
    listeningMode: settings.listeningMode,
    lastSpokenText: "",
    audioLevels: Array.from({ length: 30 }, () => 0),
  };
}

function isOverlayRoute() {
  return location.hash.includes("overlay-");
}

function isTauriRuntime() {
  return "__TAURI_INTERNALS__" in window;
}

function App() {
  const [settings, setSettings] = useState(defaultSettings);
  const [pages, setPages] = useState(defaultPages);
  const [currentPageIndex, setCurrentPageIndex] = useState(0);
  const [reading, setReading] = useState<ReadingState>(() => buildLocalReading(defaultPages, 0, defaultSettings));
  const [filePath, setFilePath] = useState<string | null>(null);
  const [status, setStatus] = useState("Ready");
  const [isRecording, setIsRecording] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [settingsTab, setSettingsTab] = useState<SettingsTab>("appearance");
  const [speechBackends, setSpeechBackends] = useState<SpeechBackendInfo[]>([]);
  const [audioInputs, setAudioInputs] = useState<AudioInputDevice[]>([]);
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [remoteUrl, setRemoteUrl] = useState<string | null>(null);
  const [directorUrl, setDirectorUrl] = useState<string | null>(null);
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);
  const progressRef = useRef(0);

  const currentText = pages[currentPageIndex] ?? "";
  const fileName = filePath?.split(/[\\/]/).pop()?.replace(/\.textream$/i, "") ?? "Untitled";

  const refreshReading = useCallback(async () => {
    try {
      const state = await invoke<ReadingState>("get_reading_state");
      setReading(state);
      if (state.pages.length) setPages(state.pages);
      setCurrentPageIndex(state.currentPageIndex);
    } catch {
      setReading(buildLocalReading(pages, currentPageIndex, settings));
    }
  }, [currentPageIndex, pages, settings]);

  const persistSettings = useCallback(async (next: AppSettings) => {
    setSettings(next);
    try {
      await invoke<AppSettings>("save_settings", { settings: next });
    } catch {
      // Browser preview does not expose Tauri commands.
    }
  }, []);

  const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    void persistSettings({ ...settings, [key]: value });
  };

  const openSettings = useCallback((tab: SettingsTab = "appearance") => {
    setSettingsTab(tab);
    setSettingsOpen(true);
  }, []);

  const openDocument = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Textream or PowerPoint", extensions: ["textream", "pptx", "key"] }],
    });
    if (typeof selected !== "string") return;
    const loaded = await invoke<string[]>("open_document", { path: selected });
    setPages(loaded);
    setCurrentPageIndex(0);
    setFilePath(selected.endsWith(".textream") ? selected : null);
    setReading(buildLocalReading(loaded, 0, settings));
    setStatus("Opened");
  }, [settings]);

  const saveDocument = useCallback(async (forceNewPath = false) => {
    const target =
      !forceNewPath && filePath
        ? filePath
        : await save({
            defaultPath: filePath ?? "Untitled.textream",
            filters: [{ name: "Textream", extensions: ["textream"] }],
          });
    if (!target) return;
    await invoke("save_document", { request: { path: target, pages } });
    setFilePath(target);
    setStatus("Saved");
  }, [filePath, pages]);

  const checkUpdates = useCallback(async () => {
    const result = await invoke<UpdateStatus>("check_for_updates");
    setUpdateStatus(result);
    setStatus(result.error ? "Update check failed" : result.isUpdateAvailable ? `Update ${result.latestVersion}` : "Up to date");
  }, []);

  const addPage = useCallback(() => {
    const next = [...pages, ""];
    setPages(next);
    setCurrentPageIndex(next.length - 1);
    setReading(buildLocalReading(next, next.length - 1, settings));
    void invoke("set_pages", { request: { pages: next, currentPageIndex: next.length - 1 } }).catch(() => undefined);
  }, [pages, settings]);

  useEffect(() => {
    invoke<AppSettings>("load_settings").then(setSettings).catch(() => setSettings(defaultSettings));
    invoke<SpeechBackendInfo[]>("list_speech_backends").then(setSpeechBackends).catch(() => setSpeechBackends([]));
    invoke<string[]>("get_launch_urls")
      .then((urls) => {
        const url = urls[0];
        if (!url) return;
        const parsed = new URL(url);
        const text = parsed.searchParams.get("text");
        if (text) {
          setPages([text]);
          setCurrentPageIndex(0);
        }
      })
      .catch(() => undefined);
    void refreshReading();
  }, []);

  useEffect(() => {
    if (!settingsOpen) return;
    invoke<AudioInputDevice[]>("list_audio_inputs").then(setAudioInputs).catch(() => setAudioInputs([]));
    invoke<MonitorInfo[]>("list_monitors").then(setMonitors).catch(() => setMonitors([]));
  }, [settingsOpen]);

  useEffect(() => {
    if (!settingsOpen) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setSettingsOpen(false);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [settingsOpen]);

  useEffect(() => {
    if (!isOverlayRoute()) return;
    const id = window.setInterval(() => void refreshReading(), 120);
    return () => window.clearInterval(id);
  }, [refreshReading]);

  useEffect(() => {
    if (!reading.isRunning || settings.listeningMode === "wordTracking") return;
    const id = window.setInterval(() => {
      progressRef.current += settings.scrollSpeed * 0.2;
      const offset = charOffsetForWordProgress(reading.words, progressRef.current);
      setReading((state) => ({ ...state, recognizedCharCount: offset, timerWordProgress: progressRef.current }));
      void invoke("jump_to_char", { charOffset: offset }).catch(() => undefined);
    }, 200);
    return () => window.clearInterval(id);
  }, [reading.isRunning, reading.words, settings.listeningMode, settings.scrollSpeed]);

  useEffect(() => {
    if (!isTauriRuntime()) return;
    const unlisten = listen<string>("menu-action", (event) => {
      switch (event.payload) {
        case "open":
          void openDocument();
          break;
        case "save":
          void saveDocument(false);
          break;
        case "save-as":
          void saveDocument(true);
          break;
        case "settings":
          openSettings();
          break;
        case "check-updates":
          void checkUpdates();
          break;
        case "add-page":
          addPage();
          break;
        default:
          break;
      }
    });
    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, [addPage, checkUpdates, openDocument, openSettings, saveDocument]);

  const updatePage = (value: string) => {
    const next = [...pages];
    next[currentPageIndex] = value;
    setPages(next);
    setReading(buildLocalReading(next, currentPageIndex, settings));
    void invoke("set_pages", { request: { pages: next, currentPageIndex } }).catch(() => undefined);
  };

  const goToPage = (index: number) => {
    const safeIndex = Math.max(0, Math.min(index, pages.length - 1));
    setCurrentPageIndex(safeIndex);
    setReading(buildLocalReading(pages, safeIndex, settings));
    void invoke("set_pages", { request: { pages, currentPageIndex: safeIndex } }).catch(() => undefined);
  };

  const removePage = (index: number) => {
    if (pages.length <= 1) return;
    const next = pages.filter((_, pageIndex) => pageIndex !== index);
    const nextIndex = Math.min(currentPageIndex > index ? currentPageIndex - 1 : currentPageIndex, next.length - 1);
    setPages(next);
    setCurrentPageIndex(nextIndex);
    setReading(buildLocalReading(next, nextIndex, settings));
    void invoke("set_pages", { request: { pages: next, currentPageIndex: nextIndex } }).catch(() => undefined);
  };

  const start = async () => {
    const state = await invoke<ReadingState>("start_reading", { request: { pages, currentPageIndex } });
    setReading(state);
    progressRef.current = 0;
    await invoke("show_overlay_window", { mode: settings.overlayMode });
    if (settings.listeningMode !== "classic") {
      const speech = await invoke<{ error?: string | null }>("start_speech", { text: currentText });
      if (speech.error) setStatus(speech.error);
    }
    setStatus("Reading");
  };

  const stop = async () => {
    const state = await invoke<ReadingState>("stop_reading");
    await invoke("stop_speech").catch(() => undefined);
    await invoke("close_overlay_windows").catch(() => undefined);
    setReading(state);
    setStatus("Stopped");
  };

  const toggleRecording = async () => {
    if (isRecording) {
      await invoke("stop_speech").catch(() => undefined);
      setIsRecording(false);
      setStatus("Dictation stopped");
      return;
    }
    const speech = await invoke<{ error?: string | null }>("start_speech", { text: currentText });
    if (speech.error) {
      setStatus(speech.error);
      return;
    }
    setIsRecording(true);
    setStatus("Dictating");
  };

  const toggleRemote = async () => {
    if (settings.browserServerEnabled) {
      await invoke("stop_remote_server");
      setRemoteUrl(null);
      await persistSettings({ ...settings, browserServerEnabled: false });
      return;
    }
    const port = await invoke<number>("start_remote_server");
    setRemoteUrl(`http://localhost:${port}`);
    await persistSettings({ ...settings, browserServerEnabled: true });
  };

  const toggleDirector = async () => {
    if (settings.directorModeEnabled) {
      await invoke("stop_director_server");
      setDirectorUrl(null);
      await persistSettings({ ...settings, directorModeEnabled: false });
      return;
    }
    const [port] = await invoke<[number, string]>("start_director_server");
    setDirectorUrl(`http://localhost:${port}`);
    await persistSettings({ ...settings, directorModeEnabled: true });
  };

  if (isOverlayRoute()) {
    return <OverlayView reading={reading} settings={settings} />;
  }

  return (
    <main className="main-window">
      <aside className="page-sidebar">
        <div className="sidebar-header">
          <button className="icon-button subtle" title="Settings" onClick={() => openSettings()}>
            <Settings size={18} />
          </button>
        </div>
        <div className="page-list">
          {pages.map((page, index) => (
            <div className={`page-row ${index === currentPageIndex ? "active" : ""}`} key={index}>
              <button className="page-select" onClick={() => goToPage(index)}>
                <span className={`page-number ${reading.readPages.includes(index) ? "read" : ""}`}>{index + 1}</span>
                <span>{pagePreview(page)}</span>
              </button>
              {pages.length > 1 ? (
                <button className="page-delete" title="Delete page" onClick={() => removePage(index)}>
                  <Trash2 size={14} />
                </button>
              ) : null}
            </div>
          ))}
        </div>
        <button className="add-page-button" onClick={addPage}>
          <Plus size={18} />
          Add Page
        </button>
      </aside>

      <section className="editor-detail">
        <header className="editor-toolbar">
          <button className="toolbar-text" onClick={() => void openDocument()}>{fileName}</button>
          <button className="toolbar-text" onClick={addPage}>
            <Plus size={16} /> Page
          </button>
          <button className="toolbar-text language-button" onClick={() => openSettings("guidance")}>
            <Languages size={16} />
            {settings.listeningMode === "wordTracking" ? localeLabel(settings.speechLocale) : listeningModeLabel(settings.listeningMode)}
          </button>
          <span className="toolbar-status">{status}</span>
        </header>

        <div className="editor-stage">
          <textarea
            className="script-editor"
            value={currentText}
            onChange={(event) => updatePage(event.currentTarget.value)}
            spellCheck={false}
          />
          {isRecording ? <WaveformPill levels={reading.audioLevels} /> : null}
          <div className="primary-controls">
            <button
              className={`circle-action mic-action ${isRecording ? "recording" : ""}`}
              title={isRecording ? "Stop dictation" : "Start dictation"}
              onClick={() => void toggleRecording()}
              disabled={reading.isRunning}
            >
              {isRecording ? <Pause size={25} fill="currentColor" /> : <Mic size={28} />}
            </button>
            <button
              className={`circle-action play-action ${reading.isRunning ? "stopping" : ""}`}
              title={reading.isRunning ? "Stop teleprompter" : "Start teleprompter"}
              onClick={() => void (reading.isRunning ? stop() : start())}
              disabled={(!reading.isRunning && !pages.some((page) => page.trim())) || isRecording}
            >
              {reading.isRunning ? <CircleStop size={27} fill="currentColor" /> : <Play size={29} fill="currentColor" />}
            </button>
          </div>
        </div>
      </section>

      {settingsOpen ? (
        <SettingsDialog
          settings={settings}
          selectedTab={settingsTab}
          onSelectTab={setSettingsTab}
          onClose={() => setSettingsOpen(false)}
          onReset={() => void persistSettings(defaultSettings)}
          onChange={updateSetting}
          speechBackends={speechBackends}
          audioInputs={audioInputs}
          monitors={monitors}
          remoteUrl={remoteUrl}
          directorUrl={directorUrl}
          onToggleRemote={() => void toggleRemote()}
          onToggleDirector={() => void toggleDirector()}
        />
      ) : null}

      {updateStatus?.isUpdateAvailable && updateStatus.releaseUrl ? (
        <button className="update-banner" onClick={() => void openUrl(updateStatus.releaseUrl!)}>
          Update {updateStatus.latestVersion}
          <ExternalLink size={15} />
        </button>
      ) : null}
    </main>
  );
}

function SettingsDialog({
  settings,
  selectedTab,
  onSelectTab,
  onClose,
  onReset,
  onChange,
  speechBackends,
  audioInputs,
  monitors,
  remoteUrl,
  directorUrl,
  onToggleRemote,
  onToggleDirector,
}: {
  settings: AppSettings;
  selectedTab: SettingsTab;
  onSelectTab: (tab: SettingsTab) => void;
  onClose: () => void;
  onReset: () => void;
  onChange: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;
  speechBackends: SpeechBackendInfo[];
  audioInputs: AudioInputDevice[];
  monitors: MonitorInfo[];
  remoteUrl: string | null;
  directorUrl: string | null;
  onToggleRemote: () => void;
  onToggleDirector: () => void;
}) {
  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <section className="settings-dialog" role="dialog" aria-modal="true" aria-label="Settings" onMouseDown={(event) => event.stopPropagation()}>
        <aside className="settings-sidebar">
          <div className="settings-heading">Settings</div>
          <nav>
            {settingTabs.map((tab) => (
              <button className={selectedTab === tab.id ? "active" : ""} key={tab.id} onClick={() => onSelectTab(tab.id)}>
                {tab.icon}
                <span>{tab.label}</span>
              </button>
            ))}
          </nav>
        </aside>

        <div className="settings-main">
          <header className="settings-mobile-header">
            <strong>{settingTabs.find((tab) => tab.id === selectedTab)?.label}</strong>
            <button className="icon-button" onClick={onClose} title="Close settings"><X size={18} /></button>
          </header>
          <div className="settings-scroll">
            {selectedTab === "appearance" ? <AppearanceSettings settings={settings} onChange={onChange} /> : null}
            {selectedTab === "guidance" ? (
              <GuidanceSettings settings={settings} onChange={onChange} speechBackends={speechBackends} audioInputs={audioInputs} />
            ) : null}
            {selectedTab === "teleprompter" ? <TeleprompterSettings settings={settings} onChange={onChange} monitors={monitors} /> : null}
            {selectedTab === "external" ? <ExternalSettings settings={settings} onChange={onChange} monitors={monitors} /> : null}
            {selectedTab === "remote" ? (
              <ServiceSettings title="Enable Remote Connection" enabled={settings.browserServerEnabled} url={remoteUrl} onToggle={onToggleRemote} />
            ) : null}
            {selectedTab === "director" ? (
              <ServiceSettings title="Enable Director Mode" enabled={settings.directorModeEnabled} url={directorUrl} onToggle={onToggleDirector} />
            ) : null}
          </div>
          <footer className="settings-footer">
            <button className="reset-button" onClick={onReset}><RotateCcw size={17} /> Reset All</button>
            <button className="done-button" onClick={onClose}>Done</button>
          </footer>
        </div>
      </section>
    </div>
  );
}

function AppearanceSettings({ settings, onChange }: SettingsSectionProps) {
  return (
    <div className="settings-content">
      <SettingGroup title="Font">
        <div className="choice-grid four-columns">
          {(["sans", "serif", "mono", "dyslexia"] as const).map((font) => (
            <button className={`font-choice ${settings.fontFamilyPreset === font ? "active" : ""}`} key={font} onClick={() => onChange("fontFamilyPreset", font)}>
              <span style={{ fontFamily: fontFamily(font) }}>Ag</span>
              <small>{font === "dyslexia" ? "Dyslexia" : capitalize(font)}</small>
            </button>
          ))}
        </div>
      </SettingGroup>
      <SettingGroup title="Size">
        <div className="choice-grid four-columns">
          {(["xs", "sm", "lg", "xl"] as const).map((size) => (
            <button className={`size-choice ${settings.fontSizePreset === size ? "active" : ""}`} key={size} onClick={() => onChange("fontSizePreset", size)}>
              <span>Ag</span><small>{size.toUpperCase()}</small>
            </button>
          ))}
        </div>
      </SettingGroup>
      <Divider />
      <ColorChoices title="Highlight Color" value={settings.fontColorPreset} onChange={(value) => onChange("fontColorPreset", value)} />
      <ColorChoices title="Cue Color" value={settings.cueColorPreset} onChange={(value) => onChange("cueColorPreset", value)} />
      <SettingGroup title="Cue Brightness">
        <Segmented value={settings.cueBrightness} options={[["dim", "Dim"], ["low", "Low"], ["medium", "Medium"], ["bright", "Bright"]]} onChange={(value) => onChange("cueBrightness", value as AppSettings["cueBrightness"])} />
      </SettingGroup>
      <Divider />
      <SettingGroup title="Dimensions">
        <RangeRow label="Width" value={settings.notchWidth} min={310} max={500} suffix="px" onChange={(value) => onChange("notchWidth", value)} />
        <RangeRow label="Height" value={settings.textAreaHeight} min={100} max={400} suffix="px" onChange={(value) => onChange("textAreaHeight", value)} />
      </SettingGroup>
    </div>
  );
}

function GuidanceSettings({ settings, onChange, speechBackends, audioInputs }: SettingsSectionProps & { speechBackends: SpeechBackendInfo[]; audioInputs: AudioInputDevice[] }) {
  return (
    <div className="settings-content">
      <Segmented value={settings.listeningMode} options={[["wordTracking", "Word Tracking"], ["classic", "Classic"], ["silencePaused", "Voice-Activated"]]} onChange={(value) => onChange("listeningMode", value as ListeningMode)} />
      <Divider />
      <SettingGroup title="Speech Language">
        <select value={settings.speechLocale} onChange={(event) => onChange("speechLocale", event.currentTarget.value)}>
          <option value="en-US">English (United States)</option>
          <option value="zh-CN">Chinese (China mainland)</option>
          <option value="zh-TW">Chinese (Taiwan)</option>
          <option value="ja-JP">Japanese</option>
          <option value="ko-KR">Korean</option>
          <option value="es-ES">Spanish</option>
          <option value="fr-FR">French</option>
          <option value="de-DE">German</option>
        </select>
      </SettingGroup>
      <Divider />
      <SettingGroup title="Speech Backend">
        <select value={settings.speechBackend} onChange={(event) => onChange("speechBackend", event.currentTarget.value as SpeechBackendKind)}>
          {speechBackends.length ? speechBackends.map((backend) => <option key={backend.id} value={backend.id} disabled={!backend.available}>{backend.label}</option>) : <option value="windowsNative">Windows Native</option>}
        </select>
      </SettingGroup>
      <SettingGroup title="Microphone">
        <select value={settings.selectedMicUid || "default"} onChange={(event) => onChange("selectedMicUid", event.currentTarget.value === "default" ? "" : event.currentTarget.value)}>
          {audioInputs.length ? audioInputs.map((input) => <option key={input.id} value={input.id}>{input.name}</option>) : <option value="default">Default microphone</option>}
        </select>
      </SettingGroup>
      {settings.listeningMode !== "wordTracking" ? <RangeRow label="Scroll Speed" value={settings.scrollSpeed} min={0.5} max={8} step={0.5} suffix=" w/s" onChange={(value) => onChange("scrollSpeed", value)} /> : null}
    </div>
  );
}

function TeleprompterSettings({ settings, onChange, monitors }: SettingsSectionProps & { monitors: MonitorInfo[] }) {
  return (
    <div className="settings-content">
      <Segmented value={settings.overlayMode} options={[["pinned", "Pinned to Top"], ["floating", "Floating Window"], ["fullscreen", "Fullscreen"]]} onChange={(value) => onChange("overlayMode", value as OverlayMode)} />
      <Divider />
      {settings.overlayMode === "pinned" ? (
        <SettingGroup title="Display">
          <Segmented value={settings.notchDisplayMode} options={[["followMouse", "Follow Mouse"], ["fixedDisplay", "Fixed Display"]]} onChange={(value) => onChange("notchDisplayMode", value as AppSettings["notchDisplayMode"])} />
        </SettingGroup>
      ) : null}
      {settings.notchDisplayMode === "fixedDisplay" || settings.overlayMode === "fullscreen" ? (
        <MonitorSelect value={settings.overlayMode === "fullscreen" ? settings.fullscreenScreenId : settings.pinnedScreenId} monitors={monitors} onChange={(value) => onChange(settings.overlayMode === "fullscreen" ? "fullscreenScreenId" : "pinnedScreenId", value)} />
      ) : null}
      <Divider />
      <ToggleRow label="Transparency" checked={settings.overlayTransparency} onChange={(value) => onChange("overlayTransparency", value)} />
      {settings.overlayTransparency ? <RangeRow label="Opacity" value={settings.overlayTransparencyOpacity} min={0.1} max={1} step={0.05} onChange={(value) => onChange("overlayTransparencyOpacity", value)} /> : null}
      <Divider />
      <CheckRow label="Elapsed Time" checked={settings.showElapsedTime} onChange={(value) => onChange("showElapsedTime", value)} />
      <CheckRow label="Hide from Screen Sharing" checked={settings.hideFromScreenShare} onChange={(value) => onChange("hideFromScreenShare", value)} />
      <Divider />
      <SettingGroup title="Pagination">
        <CheckRow label="Auto Next Page" checked={settings.autoNextPage} onChange={(value) => onChange("autoNextPage", value)} />
        {settings.autoNextPage ? <RangeRow label="Delay" value={settings.autoNextPageDelay} min={1} max={10} suffix="s" onChange={(value) => onChange("autoNextPageDelay", value)} /> : null}
      </SettingGroup>
    </div>
  );
}

function ExternalSettings({ settings, onChange, monitors }: SettingsSectionProps & { monitors: MonitorInfo[] }) {
  return (
    <div className="settings-content">
      <Segmented value={settings.externalDisplayMode} options={[["off", "Off"], ["teleprompter", "Teleprompter"], ["mirror", "Mirror"]]} onChange={(value) => onChange("externalDisplayMode", value as AppSettings["externalDisplayMode"])} />
      {settings.externalDisplayMode !== "off" ? (
        <>
          <Divider />
          <MonitorSelect value={settings.externalScreenId} monitors={monitors} onChange={(value) => onChange("externalScreenId", value)} />
        </>
      ) : null}
      {settings.externalDisplayMode === "mirror" ? (
        <SettingGroup title="Mirror Axis">
          <Segmented value={settings.mirrorAxis} options={[["horizontal", "Horizontal"], ["vertical", "Vertical"], ["both", "Both"]]} onChange={(value) => onChange("mirrorAxis", value as AppSettings["mirrorAxis"])} />
        </SettingGroup>
      ) : null}
    </div>
  );
}

function ServiceSettings({ title, enabled, url, onToggle }: { title: string; enabled: boolean; url: string | null; onToggle: () => void }) {
  return (
    <div className="settings-content service-settings">
      <ToggleRow label={title} checked={enabled} onChange={onToggle} />
      {enabled && url ? (
        <div className="service-address">
          <code>{url}</code>
          <button className="icon-button" title="Copy address" onClick={() => void navigator.clipboard.writeText(url)}><Copy size={17} /></button>
          <button className="icon-button" title="Open in browser" onClick={() => void openUrl(url)}><ExternalLink size={17} /></button>
        </div>
      ) : null}
    </div>
  );
}

interface SettingsSectionProps {
  settings: AppSettings;
  onChange: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;
}

function SettingGroup({ title, children }: { title: string; children: ReactNode }) {
  return <section className="setting-group"><h2>{title}</h2>{children}</section>;
}

function Divider() {
  return <div className="settings-divider" />;
}

function Segmented({ value, options, onChange }: { value: string; options: [string, string][]; onChange: (value: string) => void }) {
  return (
    <div className="segmented-control">
      {options.map(([id, label]) => <button className={value === id ? "active" : ""} key={id} onClick={() => onChange(id)}>{label}</button>)}
    </div>
  );
}

function ColorChoices({ title, value, onChange }: { title: string; value: FontColorPreset; onChange: (value: FontColorPreset) => void }) {
  return (
    <SettingGroup title={title}>
      <div className="color-options">
        {(Object.keys(colorMap) as FontColorPreset[]).map((color) => (
          <button className={value === color ? "active" : ""} key={color} onClick={() => onChange(color)}>
            <span style={{ backgroundColor: colorMap[color] }}>{value === color ? <Check size={18} /> : null}</span>
            <small>{capitalize(color)}</small>
          </button>
        ))}
      </div>
    </SettingGroup>
  );
}

function ToggleRow({ label, checked, onChange }: { label: string; checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <label className="toggle-row">
      <strong>{label}</strong>
      <input type="checkbox" checked={checked} onChange={(event) => onChange(event.currentTarget.checked)} />
      <span className="toggle-track"><span /></span>
    </label>
  );
}

function CheckRow({ label, checked, onChange }: { label: string; checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <label className="check-row">
      <input type="checkbox" checked={checked} onChange={(event) => onChange(event.currentTarget.checked)} />
      <span>{checked ? <Check size={16} /> : null}</span>
      <strong>{label}</strong>
    </label>
  );
}

function RangeRow({ label, value, min, max, step = 1, suffix = "", onChange }: { label: string; value: number; min: number; max: number; step?: number; suffix?: string; onChange: (value: number) => void }) {
  return (
    <label className="range-row">
      <span><strong>{label}</strong><code>{value}{suffix}</code></span>
      <input type="range" value={value} min={min} max={max} step={step} onChange={(event) => onChange(Number(event.currentTarget.value))} />
    </label>
  );
}

function MonitorSelect({ value, monitors, onChange }: { value: number; monitors: MonitorInfo[]; onChange: (value: number) => void }) {
  return (
    <SettingGroup title="Target Display">
      <select value={value} onChange={(event) => onChange(Number(event.currentTarget.value))}>
        {monitors.length ? monitors.map((monitor) => <option value={monitor.id} key={monitor.id}>{monitor.name || `Display ${monitor.id + 1}`} ({monitor.width} x {monitor.height})</option>) : <option value={0}>Primary display</option>}
      </select>
    </SettingGroup>
  );
}

function WaveformPill({ levels }: { levels: number[] }) {
  return <div className="waveform-pill"><Waveform levels={levels} /></div>;
}

function PrompterView({ reading, settings }: { reading: ReadingState; settings: AppSettings }) {
  const words = reading.words;
  const fontSize = fontSizeMap[settings.fontSizePreset];
  let offset = 0;
  return (
    <div className="prompter" style={{ minHeight: settings.textAreaHeight, fontSize, fontFamily: fontFamily(settings.fontFamilyPreset) }}>
      <div className="elapsed">{settings.showElapsedTime && reading.isRunning ? "LIVE" : ""}</div>
      <div className="word-flow">
        {words.map((word, index) => {
          const start = offset;
          offset += [...word].length + 1;
          const read = start < reading.recognizedCharCount;
          const annotation = word.startsWith("[") && word.endsWith("]");
          return <button key={`${word}-${index}-${start}`} className={`${read ? "read" : ""} ${annotation ? "annotation" : ""}`} onClick={() => void invoke("jump_to_char", { charOffset: start }).catch(() => undefined)} style={{ color: read ? colorMap[settings.fontColorPreset] : undefined }}>{word}</button>;
        })}
      </div>
      <Waveform levels={reading.audioLevels} />
      {reading.totalCharCount > 0 && reading.recognizedCharCount >= reading.totalCharCount ? <div className="done"><CheckCircle2 size={26} /> Done</div> : null}
    </div>
  );
}

function OverlayView({ reading, settings }: { reading: ReadingState; settings: AppSettings }) {
  return <main className={`overlay-shell ${location.hash.includes("fullscreen") ? "fullscreen" : ""}`}><PrompterView reading={reading} settings={settings} /></main>;
}

function Waveform({ levels }: { levels: number[] }) {
  return <div className="waveform">{levels.map((level, index) => <span key={index} style={{ height: `${Math.max(3, level * 30)}px` }} />)}</div>;
}

function pagePreview(page: string) {
  const words = page.trim().split(/\s+/).filter(Boolean).slice(0, 5).join(" ");
  if (!words) return "Empty";
  return words.length > 30 ? `${words.slice(0, 30)}...` : words;
}

function localeLabel(locale: string) {
  const labels: Record<string, string> = {
    "en-US": "English (United States)",
    "zh-CN": "Chinese (China mainland)",
    "zh-TW": "Chinese (Taiwan)",
    "ja-JP": "Japanese",
    "ko-KR": "Korean",
  };
  return labels[locale] ?? locale;
}

function listeningModeLabel(mode: ListeningMode) {
  return mode === "classic" ? "Classic" : mode === "silencePaused" ? "Voice-Activated" : "Word Tracking";
}

function fontFamily(value: AppSettings["fontFamilyPreset"]) {
  switch (value) {
    case "serif": return "Georgia, Cambria, serif";
    case "mono": return "ui-monospace, SFMono-Regular, Consolas, monospace";
    case "dyslexia": return "OpenDyslexic, Atkinson Hyperlegible, Arial, sans-serif";
    default: return "Segoe UI, Inter, system-ui, sans-serif";
  }
}

function capitalize(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}

export default App;
