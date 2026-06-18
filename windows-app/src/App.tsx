import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  CheckCircle2,
  FileDown,
  FileUp,
  Mic,
  Monitor,
  Pause,
  Play,
  Radio,
  Save,
  Settings,
  Square,
  Tv,
} from "lucide-react";
import "./App.css";
import {
  AppSettings,
  FontColorPreset,
  ListeningMode,
  OverlayMode,
  ReadingState,
  SpeechBackendInfo,
  SpeechBackendKind,
  UpdateStatus,
  colorMap,
  fontSizeMap,
} from "./shared/types";
import { buildWordItems, charOffsetForWordProgress, splitTextIntoWords } from "./shared/text";

const defaultPages = [
  `Welcome to Textream for Windows. This is a teleprompter workspace for scripts, overlays, remote viewing, and director control. [pause]

Start reading to open the overlay. Classic mode scrolls at a constant pace, while word tracking is wired through the selected speech backend.`,
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

function App() {
  const [settings, setSettings] = useState(defaultSettings);
  const [pages, setPages] = useState(defaultPages);
  const [currentPageIndex, setCurrentPageIndex] = useState(0);
  const [reading, setReading] = useState<ReadingState>(() =>
    buildLocalReading(defaultPages, 0, defaultSettings),
  );
  const [filePath, setFilePath] = useState<string | null>(null);
  const [status, setStatus] = useState("Ready");
  const [speechBackends, setSpeechBackends] = useState<SpeechBackendInfo[]>([]);
  const [remoteUrl, setRemoteUrl] = useState<string | null>(null);
  const [directorUrl, setDirectorUrl] = useState<string | null>(null);
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);
  const progressRef = useRef(0);

  const currentText = pages[currentPageIndex] ?? "";
  const currentWords = useMemo(() => splitTextIntoWords(currentText), [currentText]);

  const refreshReading = useCallback(async () => {
    try {
      const state = await invoke<ReadingState>("get_reading_state");
      setReading(state);
      setPages(state.pages.length ? state.pages : pages);
      setCurrentPageIndex(state.currentPageIndex);
    } catch {
      setReading(buildLocalReading(pages, currentPageIndex, settings));
    }
  }, [currentPageIndex, pages, settings]);

  useEffect(() => {
    invoke<AppSettings>("load_settings")
      .then(setSettings)
      .catch(() => setSettings(defaultSettings));
    invoke<SpeechBackendInfo[]>("list_speech_backends")
      .then(setSpeechBackends)
      .catch(() => setSpeechBackends([]));
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
    refreshReading();
  }, []);

  useEffect(() => {
    if (!isOverlayRoute()) return;
    const id = window.setInterval(refreshReading, 120);
    return () => window.clearInterval(id);
  }, [refreshReading]);

  useEffect(() => {
    if (!reading.isRunning || settings.listeningMode === "wordTracking") return;
    const id = window.setInterval(() => {
      progressRef.current += settings.scrollSpeed * 0.2;
      const offset = charOffsetForWordProgress(reading.words, progressRef.current);
      setReading((state) => ({ ...state, recognizedCharCount: offset, timerWordProgress: progressRef.current }));
      invoke("jump_to_char", { charOffset: offset }).catch(() => undefined);
    }, 200);
    return () => window.clearInterval(id);
  }, [reading.isRunning, reading.words, settings.listeningMode, settings.scrollSpeed]);

  const persistSettings = async (next: AppSettings) => {
    setSettings(next);
    await invoke<AppSettings>("save_settings", { settings: next });
  };

  const updatePage = (value: string) => {
    const next = [...pages];
    next[currentPageIndex] = value;
    setPages(next);
    invoke("set_pages", { request: { pages: next, currentPageIndex } }).catch(() => undefined);
  };

  const openDocument = async () => {
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
  };

  const saveDocument = async () => {
    const target =
      filePath ??
      (await save({
        defaultPath: "Untitled.textream",
        filters: [{ name: "Textream", extensions: ["textream"] }],
      }));
    if (!target) return;
    await invoke("save_document", { request: { path: target, pages } });
    setFilePath(target);
    setStatus("Saved");
  };

  const start = async () => {
    const state = await invoke<ReadingState>("start_reading", {
      request: { pages, currentPageIndex },
    });
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

  const goToPage = (index: number) => {
    const safeIndex = Math.max(0, Math.min(index, pages.length - 1));
    setCurrentPageIndex(safeIndex);
    setReading(buildLocalReading(pages, safeIndex, settings));
    invoke("set_pages", { request: { pages, currentPageIndex: safeIndex } }).catch(() => undefined);
  };

  const addPage = () => {
    const next = [...pages, ""];
    setPages(next);
    setCurrentPageIndex(next.length - 1);
  };

  const startRemote = async () => {
    const port = await invoke<number>("start_remote_server");
    const next = { ...settings, browserServerEnabled: true };
    await persistSettings(next);
    setRemoteUrl(`http://localhost:${port}`);
  };

  const stopRemote = async () => {
    await invoke("stop_remote_server");
    const next = { ...settings, browserServerEnabled: false };
    await persistSettings(next);
    setRemoteUrl(null);
  };

  const startDirector = async () => {
    const [port] = await invoke<[number, string]>("start_director_server");
    const next = { ...settings, directorModeEnabled: true };
    await persistSettings(next);
    setDirectorUrl(`http://localhost:${port}`);
  };

  const stopDirector = async () => {
    await invoke("stop_director_server");
    const next = { ...settings, directorModeEnabled: false };
    await persistSettings(next);
    setDirectorUrl(null);
  };

  const checkUpdates = async () => {
    const result = await invoke<UpdateStatus>("check_for_updates");
    setUpdateStatus(result);
  };

  if (isOverlayRoute()) {
    return <OverlayView reading={reading} settings={settings} />;
  }

  return (
    <main className="app-shell">
      <section className="editor-pane">
        <header className="topbar">
          <div>
            <h1>Textream</h1>
            <p>{status}</p>
          </div>
          <div className="topbar-actions">
            <button title="Open" onClick={openDocument}>
              <FileUp size={18} />
            </button>
            <button title="Save" onClick={saveDocument}>
              <Save size={18} />
            </button>
            <button title="Check for updates" onClick={checkUpdates}>
              <FileDown size={18} />
            </button>
          </div>
        </header>

        <textarea
          className="script-editor"
          value={currentText}
          onChange={(event) => updatePage(event.currentTarget.value)}
          spellCheck={false}
        />

        <footer className="pagebar">
          <button onClick={() => goToPage(currentPageIndex - 1)} disabled={currentPageIndex === 0}>
            Prev
          </button>
          <div className="page-tabs">
            {pages.map((page, index) => (
              <button
                className={index === currentPageIndex ? "active" : ""}
                key={index}
                onClick={() => goToPage(index)}
              >
                {index + 1}
                {page.trim() ? "" : "*"}
              </button>
            ))}
          </div>
          <button onClick={addPage}>Add</button>
          <button onClick={() => goToPage(currentPageIndex + 1)} disabled={currentPageIndex >= pages.length - 1}>
            Next
          </button>
        </footer>
      </section>

      <section className="preview-pane">
        <PrompterView reading={reading} fallbackWords={currentWords} settings={settings} />
        <div className="transport">
          <button className="primary-action" onClick={reading.isRunning ? stop : start}>
            {reading.isRunning ? <Square size={18} /> : <Play size={18} />}
            {reading.isRunning ? "Stop" : "Start"}
          </button>
          <button
            onClick={() => invoke("start_speech", { text: currentText }).catch((error) => setStatus(String(error)))}
            disabled={settings.listeningMode === "classic"}
          >
            <Mic size={18} />
            Mic
          </button>
          <button onClick={() => invoke("stop_speech").catch(() => undefined)}>
            <Pause size={18} />
            Mute
          </button>
        </div>
      </section>

      <aside className="settings-pane">
        <PanelTitle icon={<Settings size={18} />} label="Settings" />
        <Field label="Mode">
          <Segmented
            value={settings.listeningMode}
            options={[
              ["wordTracking", "Word"],
              ["classic", "Classic"],
              ["silencePaused", "Voice"],
            ]}
            onChange={(value) => persistSettings({ ...settings, listeningMode: value as ListeningMode })}
          />
        </Field>
        <Field label="Overlay">
          <Segmented
            value={settings.overlayMode}
            options={[
              ["pinned", "Top"],
              ["floating", "Float"],
              ["fullscreen", "Full"],
            ]}
            onChange={(value) => persistSettings({ ...settings, overlayMode: value as OverlayMode })}
          />
        </Field>
        <Field label="Speed">
          <input
            type="range"
            min="0.5"
            max="8"
            step="0.5"
            value={settings.scrollSpeed}
            onChange={(event) => persistSettings({ ...settings, scrollSpeed: Number(event.currentTarget.value) })}
          />
          <span>{settings.scrollSpeed.toFixed(1)}</span>
        </Field>
        <Field label="Size">
          <input
            type="range"
            min="310"
            max="500"
            value={settings.notchWidth}
            onChange={(event) => persistSettings({ ...settings, notchWidth: Number(event.currentTarget.value) })}
          />
        </Field>
        <Field label="Text height">
          <input
            type="range"
            min="100"
            max="400"
            value={settings.textAreaHeight}
            onChange={(event) => persistSettings({ ...settings, textAreaHeight: Number(event.currentTarget.value) })}
          />
        </Field>
        <Field label="Font">
          <Segmented
            value={settings.fontSizePreset}
            options={[
              ["xs", "XS"],
              ["sm", "SM"],
              ["lg", "LG"],
              ["xl", "XL"],
            ]}
            onChange={(value) => persistSettings({ ...settings, fontSizePreset: value as AppSettings["fontSizePreset"] })}
          />
        </Field>
        <Field label="Highlight">
          <div className="swatches">
            {(Object.keys(colorMap) as FontColorPreset[]).map((color) => (
              <button
                key={color}
                title={color}
                className={settings.fontColorPreset === color ? "active" : ""}
                style={{ background: colorMap[color] }}
                onClick={() => persistSettings({ ...settings, fontColorPreset: color })}
              />
            ))}
          </div>
        </Field>
        <Field label="Speech">
          <select
            value={settings.speechBackend}
            onChange={(event) =>
              persistSettings({ ...settings, speechBackend: event.currentTarget.value as SpeechBackendKind })
            }
          >
            {speechBackends.map((backend) => (
              <option key={backend.id} value={backend.id} disabled={!backend.available}>
                {backend.label}
              </option>
            ))}
          </select>
        </Field>

        <PanelTitle icon={<Radio size={18} />} label="Remote" />
        <ServiceRow active={Boolean(remoteUrl)} label={remoteUrl ?? `:${settings.browserServerPort}`} onStart={startRemote} onStop={stopRemote} />
        <PanelTitle icon={<Tv size={18} />} label="Director" />
        <ServiceRow active={Boolean(directorUrl)} label={directorUrl ?? `:${settings.directorServerPort}`} onStart={startDirector} onStop={stopDirector} />
        <PanelTitle icon={<Monitor size={18} />} label="System" />
        <button onClick={() => invoke("show_overlay_window", { mode: settings.overlayMode })}>Open overlay</button>
        <button onClick={() => invoke("close_overlay_windows")}>Close overlays</button>
        {updateStatus ? (
          <p className="update-status">
            {updateStatus.error
              ? updateStatus.error
              : updateStatus.isUpdateAvailable
                ? `Update ${updateStatus.latestVersion}`
                : "Up to date"}
          </p>
        ) : null}
      </aside>
    </main>
  );
}

function PanelTitle({ icon, label }: { icon: ReactNode; label: string }) {
  return (
    <div className="panel-title">
      {icon}
      <span>{label}</span>
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="field">
      <span>{label}</span>
      <div>{children}</div>
    </label>
  );
}

function Segmented({
  value,
  options,
  onChange,
}: {
  value: string;
  options: [string, string][];
  onChange: (value: string) => void;
}) {
  return (
    <div className="segmented">
      {options.map(([id, label]) => (
        <button key={id} className={value === id ? "active" : ""} onClick={() => onChange(id)}>
          {label}
        </button>
      ))}
    </div>
  );
}

function ServiceRow({
  active,
  label,
  onStart,
  onStop,
}: {
  active: boolean;
  label: string;
  onStart: () => void;
  onStop: () => void;
}) {
  return (
    <div className="service-row">
      <span className={active ? "service-dot active" : "service-dot"} />
      <span>{label}</span>
      <button onClick={active ? onStop : onStart}>{active ? "Stop" : "Start"}</button>
    </div>
  );
}

function PrompterView({
  reading,
  fallbackWords,
  settings,
}: {
  reading: ReadingState;
  fallbackWords: string[];
  settings: AppSettings;
}) {
  const words = reading.words.length ? reading.words : fallbackWords;
  const fontSize = fontSizeMap[settings.fontSizePreset];
  let offset = 0;
  return (
    <div
      className="prompter"
      style={{
        minHeight: settings.textAreaHeight,
        fontSize,
        fontFamily: fontFamily(settings.fontFamilyPreset),
      }}
    >
      <div className="elapsed">{settings.showElapsedTime && reading.isRunning ? "LIVE" : "IDLE"}</div>
      <div className="word-flow">
        {words.map((word, index) => {
          const start = offset;
          offset += [...word].length + 1;
          const read = start < reading.recognizedCharCount;
          const annotation = word.startsWith("[") && word.endsWith("]");
          return (
            <button
              key={`${word}-${index}-${start}`}
              className={[read ? "read" : "", annotation ? "annotation" : ""].join(" ")}
              onClick={() => invoke("jump_to_char", { charOffset: start }).catch(() => undefined)}
              style={{
                color: read ? colorMap[settings.fontColorPreset] : undefined,
              }}
            >
              {word}
            </button>
          );
        })}
      </div>
      <Waveform levels={reading.audioLevels} />
      {reading.totalCharCount > 0 && reading.recognizedCharCount >= reading.totalCharCount ? (
        <div className="done">
          <CheckCircle2 size={26} />
          Done
        </div>
      ) : null}
    </div>
  );
}

function OverlayView({ reading, settings }: { reading: ReadingState; settings: AppSettings }) {
  return (
    <main className={`overlay-shell ${location.hash.includes("fullscreen") ? "fullscreen" : ""}`}>
      <PrompterView reading={reading} fallbackWords={[]} settings={settings} />
    </main>
  );
}

function Waveform({ levels }: { levels: number[] }) {
  return (
    <div className="waveform">
      {levels.map((level, index) => (
        <span key={index} style={{ height: `${Math.max(3, level * 30)}px` }} />
      ))}
    </div>
  );
}

function fontFamily(value: AppSettings["fontFamilyPreset"]) {
  switch (value) {
    case "serif":
      return "Georgia, Cambria, serif";
    case "mono":
      return "ui-monospace, SFMono-Regular, Consolas, monospace";
    case "dyslexia":
      return "OpenDyslexic, Atkinson Hyperlegible, Arial, sans-serif";
    default:
      return "Inter, Segoe UI, system-ui, sans-serif";
  }
}

export default App;
