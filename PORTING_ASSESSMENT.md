# Textream Windows Porting Assessment

This table maps the macOS SwiftUI/AppKit implementation to a Windows-first Tauri + Vite + React implementation. Porting levels are:

- `无损移植`: keep behavior and data model with direct TypeScript/Rust implementation.
- `微调写法`: keep behavior but swap library/runtime details.
- `改变实现方式`: keep user-facing intent with a platform-specific implementation.
- `彻底改变逻辑`: no true Windows equivalent; replace with a Windows-native interpretation.

| 功能单元 | mac 源文件/符号 | 平台依赖 | Windows/Tauri 替代方案 | 移植等级 | 实现动作 | 验证方式 |
|---|---|---|---|---|---|---|
| 应用入口与主窗口 | `TextreamApp`, `AppDelegate` | SwiftUI scene, `NSApplicationDelegate`, macOS menu commands | Tauri `main` window, React routes/state, Rust setup hooks | 改变实现方式 | 创建 `windows-app`，用 React 主界面承载编辑器和设置页，Rust 注册命令和事件 | `npm run build`, Tauri dev window starts |
| `.textream` 文档格式 | `TextreamService.saveToURL`, `openFileAtURL` | `NSSavePanel`, `NSOpenPanel`, `NSDocumentController` | Tauri dialog/fs commands + Rust JSON read/write | 无损移植 | 继续保存 `[String]` JSON，命令为 `open_document`/`save_document` | Rust unit test + 手动打开保存 |
| 多页状态与跳页 | `pages`, `currentPageIndex`, `advanceToNextPage`, `jumpToPage` | Swift `@Published`, main-thread UI mutation | React reducer/store + Rust reading state commands | 无损移植 | 保留跳过空页、已读页、页预览和下一页判断 | 前端测试页切换和空页跳过 |
| 文本拆词 | `splitTextIntoWords`, `Unicode.Scalar.isCJK` | Swift Unicode APIs | TypeScript/Rust Unicode char iteration | 无损移植 | 迁移 CJK 单字切分、空白折叠、标注 token 保留 | Rust/TS tests for English, CJK, annotations |
| 读词高亮与进度 | `SpeechRecognizer.recognizedCharCount`, `WordFlowLayout` | SwiftUI layout/preferences | React token layout + CSS wrap + char offset math | 微调写法 | 用 shared word offsets 驱动高亮和 tap-to-jump | Vitest active word/offset tests |
| 经典滚动 | `timerWordProgress`, `scrollSpeed` | SwiftUI `Timer.publish` | React timer/requestAnimationFrame | 无损移植 | 按 words/s 推进 `timerWordProgress`，支持手动滚动校准 | 前端测试和手动播放 |
| 语音触发滚动 | `ListeningMode.silencePaused`, `isSpeaking` | AVFoundation audio RMS | Speech backend audio levels + React/Rust state | 微调写法 | 根据最近音量阈值控制滚动暂停/恢复 | mock audio levels unit test |
| 逐词语音跟踪 | `SpeechRecognizer.start`, `normalize`, fuzzy matching | `SFSpeechRecognizer`, `AVAudioEngine`, macOS speech permissions | 默认 Windows native backend；接口预留 Web Speech/local model | 改变实现方式 | 用 `SpeechBackend` 抽象输出识别文本、音量、监听状态；Windows 实现在 `cfg(windows)` | Rust backend tests with mock; Windows 手测 |
| 编辑器听写 | `DictationManager`, `ContentView.startRecording` | `SFSpeechRecognizer`, `AVAudioEngine` | 同一 speech backend 的 dictation mode | 改变实现方式 | 复用语音事件，把增量文本插入当前页光标位置 | 手动听写和 mock update tests |
| 麦克风设备选择 | `AudioInputDevice.allInputDevices` | CoreAudio | Windows audio device enumeration behind Rust command | 改变实现方式 | 暴露 `list_audio_inputs`，非 Windows 返回默认占位 | Windows 手测设备列表 |
| 设置模型 | `NotchSettings` | `UserDefaults`, `NSFont`, Swift enums | Tauri store/json settings + TS enums | 微调写法 | 实现 `load_settings`/`save_settings`，保留默认值和端口 | Rust serialization tests |
| 字体与颜色预设 | `FontSizePreset`, `FontFamilyPreset`, `FontColorPreset` | `NSFont` and bundled font loading | CSS font stacks + bundled OpenDyslexic asset | 微调写法 | 迁移尺寸、颜色、字体族到 CSS variables | UI visual check |
| 主编辑器 UI | `ContentView`, `HighlightingTextEditor` | SwiftUI, `NSTextView`, drag/drop | React textarea/contenteditable + Tauri drag/drop | 改变实现方式 | 实现页面编辑、播放、录音、PPTX drop 区和 About/Settings | 前端 tests + manual drag/drop |
| 刘海固定覆盖层 | `NotchOverlayController.showPinned`, `DynamicIslandShape` | `NSPanel`, notch geometry, menu bar | Borderless always-on-top top-edge overlay window | 彻底改变逻辑 | 复刻 Dynamic Island 视觉，不依赖硬件刘海 | Tauri window positioning manual check |
| 悬浮覆盖层 | `showFloating`, `FloatingOverlayView` | `NSPanel`, cursor tracking, AppKit glass | Tauri borderless always-on-top window + CSS blur | 改变实现方式 | 实现可拖动悬浮窗、跟随光标、停止按钮 | Manual window behavior check |
| 全屏提示器 | `showFullscreen` | AppKit screen selection and Esc monitor | Tauri fullscreen window on selected monitor | 改变实现方式 | 创建 fullscreen window，监听 Esc 关闭 | Multi-monitor manual check |
| 外接屏/Sidecar | `ExternalDisplayController` | `NSScreen`, Sidecar display IDs | Tauri monitor API, Windows multi-display | 改变实现方式 | Sidecar 语义改为目标显示器，全屏输出和镜像翻转 | Windows multi-display check |
| 镜像模式 | `MirrorAxis.scaleX/scaleY` | SwiftUI transforms | CSS `transform: scaleX/scaleY` | 无损移植 | 在外接显示器视图中套用 transform | Visual check |
| 屏幕分享隐藏 | `hideFromScreenShare` window sharing type | macOS window sharing policy | Windows display affinity where available | 改变实现方式 | `cfg(windows)` 使用 capture exclusion；其他平台 no-op | Windows capture manual check |
| Remote HTTP/WS | `BrowserServer`, `BrowserState` | Network.framework, NW WebSocket | Rust HTTP + WebSocket server | 微调写法 | 保留 state JSON 字段和默认 `7373/7374` | WebSocket integration test |
| Director HTTP/WS | `DirectorServer`, `DirectorState`, `DirectorCommand` | Network.framework, `SecRandomCopyBytes` | Rust HTTP + WebSocket server + random token | 微调写法 | 保留 auth、`setText`、`updateText`、`stop` 和 10Hz broadcast | Rust command/state tests |
| PPTX notes import | `PresentationNotesExtractor` | `/usr/bin/unzip`, `XMLParser` | Rust `zip` + XML parser | 微调写法 | 解析 `ppt/notesSlides/notesSlide*.xml`，过滤占位内容 | Rust fixture/unit tests |
| Keynote import提示 | `ContentView`, `TextreamService.openFile` | Keynote/mac-only format | Windows alert explaining PPTX export required | 无损移植 | 保留 `.key` 不支持提示 | Manual unsupported file check |
| 更新检查 | `UpdateChecker` | `Bundle`, `NSAlert`, `NSWorkspace` | Rust/TS HTTP request + Tauri opener/dialog or updater | 微调写法 | 查询 GitHub latest release，显示状态 | Mocked HTTP test/manual |
| URL scheme | `Info.plist`, `handleURL` | CFBundleURLTypes, `NSApplication.open urls` | Tauri deep-link + single-instance | 改变实现方式 | 注册 `textream://read?text=...`，运行中转发到主窗口 | Tauri integration/manual |
| macOS Services | `readInTextream` | `NSServices`, pasteboard | Windows shell/send-to or clipboard command | 彻底改变逻辑 | 第一轮不做系统服务；保留粘贴/深链作为替代 | Assessment documented |
| 安装包与图标 | Xcode project, `build.sh`, assets | Xcode archive, DMG, app iconset | Tauri bundler MSI/NSIS, icon conversion | 改变实现方式 | 使用现有 PNG/SVG 生成 Tauri icons and Windows bundle metadata | `npm run tauri build` on Windows |

## Current Implementation Status

- Done: `windows-app` scaffold, React workspace UI, shared TypeScript models, Rust settings/document/reading/text/PPTX/update modules, overlay window commands, Remote and Director HTTP/WebSocket servers, and CodeGraph re-index.
- Verified on this Mac: `npm run build`, `cargo test`, and `npm run tauri build`.
- Pending Windows validation: Windows native speech backend internals, microphone device enumeration beyond the default placeholder, capture protection behavior, Windows installer output, URL scheme registration, and multi-monitor fullscreen behavior.
