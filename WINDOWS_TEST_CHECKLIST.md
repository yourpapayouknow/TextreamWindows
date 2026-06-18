# Textream Windows Test Checklist

## 1. Windows 打包前置环境

在 Windows 11 机器上准备：

- Node.js 20+ 或 22+。
- Rust stable toolchain with `rustup`。
- Microsoft C++ Build Tools / Visual Studio Build Tools，安装 `Desktop development with C++`。
- WebView2 Runtime。

验证命令：

```powershell
node -v
npm -v
rustc --version
cargo --version
rustup target list --installed
```

## 2. 打包命令

### GitHub Actions 打包

推荐使用 GitHub Actions 产物：

1. 打开 `https://github.com/yourpapayouknow/TextreamWindows/actions/workflows/release-windows.yml`。
2. 运行 `Build Windows Test Release` workflow。
3. 等 workflow 完成后，打开 `https://github.com/yourpapayouknow/TextreamWindows/releases`。
4. 下载最新 `Textream Windows Test ...` prerelease 里的 `.exe` 安装器。

### 本地 Windows 打包

从仓库根目录进入 Windows 工程：

```powershell
cd windows-app
npm install
npm run build
cd src-tauri
cargo test
cd ..
npm run tauri build
```

预期产物：

- 便携 exe：`windows-app\src-tauri\target\release\textream-windows.exe`
- 安装器：检查 `windows-app\src-tauri\target\release\bundle\` 下的 `nsis` 或 `msi` 目录。

如果只想先跑开发版：

```powershell
cd windows-app
npm run tauri dev
```

## 3. 启动与基础 UI

- 启动应用，主窗口标题应为 `Textream`。
- 左侧脚本编辑器可以输入、粘贴、多行编辑。
- 页面栏可以 `Add` 新页、切换页、上一页/下一页。
- 空页标签会显示 `*`。
- 中间预览区应实时显示当前页分词后的提示器内容。
- 设置栏不会挤压主编辑器，窗口缩小时不应出现文本重叠。

## 4. 文件功能

- 点击打开按钮，选择 `.textream` 文件，内容应按页加载。
- 点击保存按钮，保存为 `.textream`，文件内容应是 JSON 字符串数组。
- 重新打开刚保存的 `.textream`，页数和文字应一致。
- 打开 `.pptx`，应提取 presenter notes 为页面。
- 打开 `.key`，应提示需要先导出为 PowerPoint。

## 5. 阅读模式

- `Classic` 模式点击 `Start`，预览和 overlay 高亮应按设置速度推进。
- `Word` 模式点击 `Start`，应打开 overlay，并尝试启动语音后端。
- `Voice` 模式点击 `Start`，应进入语音相关阅读状态。
- 点击词语，应跳转高亮进度。
- 点击 `Stop`，overlay 关闭，阅读状态停止。
- 多页脚本中空页应被跳过。

## 6. Overlay / 窗口

- `Top` overlay：应创建顶部无边框置顶窗口。
- `Float` overlay：应创建可调整大小的置顶悬浮窗口。
- `Full` overlay：应创建全屏提示器窗口。
- `Close overlays` 应关闭所有 overlay 窗口。
- 多显示器下验证全屏窗口是否出现在预期显示器；当前实现仍需要真机调整。

## 7. 语音与麦克风

- 设置中默认语音后端应是 `Windows Native`。
- 检查 Windows 麦克风权限是否允许应用访问。
- 点击 `Mic`，确认不会崩溃；如果后端未完整接入，应显示明确错误或保持可恢复状态。
- 当前代码已有后端边界，但 Windows 原生识别 internals 仍是待验证/补全项。

## 8. Remote 模式

- 点击 Remote `Start`。
- 浏览器打开 `http://localhost:7373`。
- 应看到远程提示器页面。
- 应用点击 `Start` 后，远程页面应通过 WebSocket 更新词和高亮进度。
- 点击 Remote `Stop` 后，端口应停止服务。

## 9. Director 模式

- 点击 Director `Start`。
- 浏览器打开显示的 `http://localhost:7575`。
- 页面应显示 Director 编辑器。
- 输入脚本并点击 `Go`，应用主阅读状态应开始。
- 阅读中编辑 Director 文本，应通过 `updateText` 保留已读进度。
- 点击 `Stop`，应用阅读状态应停止。

## 10. 更新检查

- 点击更新检查按钮。
- 如果网络可访问 GitHub，应显示 `Up to date` 或新版本号。
- 断网时应显示错误文本，应用不能崩溃。

## 11. 已知待 Windows 真机确认项

- Windows native speech recognition 实际识别与音频电平。
- 麦克风设备枚举，目前非 Windows 环境只验证了默认占位。
- 屏幕捕获保护，目前命令面已存在，Windows 行为需实机验证。
- `textream://read?text=...` URL scheme 注册和单实例转发。
- NSIS/MSI 安装器是否按目标机器工具链完整生成。
