## Context

Phase 7 覆盖 7 项桌面系 WebView 功能。探查发现 3 项已实现（R85/R86/R91），2 项是平台限制（R82/R90），2 项需实现（R75/R77）。

### 当前状态

| R# | 功能 | OHOS 现状 | 代码位置 |
|----|------|-----------|----------|
| R75 | HTTPS 自定义协议 | raw scheme 已实现，无 HTTPS toggle | `wry/src/lib.rs:1934-1953`（ExtOhos 只有 `with_window_id`） |
| R77 | 窗口聚焦 | `set_focus`/`set_focusable` 是 FIXME stub | `tao/.../ohos/mod.rs:813-820` |
| R82 | 剪贴板属性 | 被静默忽略 | `wry/src/ohos/mod.rs:61-84`（`..` catch-all） |
| R85 | 数据存储标识 | 有意不使用（同 Android） | `tauri/.../manager/webview.rs:537-560` |
| R86 | 数据目录 | 完全实现 | `tauri/.../path/ohos.rs:45-128` |
| R90 | 点击穿透 | 返回 `NotSupported` | `tao/.../ohos/mod.rs:929-933` |
| R91 | 热键缩放 | JS 路径 cfg-gate 到 desktop | `tauri/.../manager/webview.rs:562-581` |

## Goals / Non-Goals

**Goals:**
- R77: 实现 Float 子窗口的 `set_focus`（通过 `openharmony-ability` 窗口 API）
- R75: 添加 `WebViewBuilderExtOhos::with_https_scheme` toggle
- R82/R85/R86/R90/R91: 在 spec 中标注状态（已实现/平台限制/设计决策），不需要代码修改
- R86: 核实 `filesDir` 双重 join 问题
- R91: 验证 OHOS 桌面端热键缩放行为

**Non-Goals:**
- 不实现 R90 点击穿透（OHOS 无窗口级 API，平台限制）
- 不修改 R82 剪贴板属性（ArkWeb 默认允许，属性无效但不影响功能）
- 不修改 R85 数据存储标识（设计决策，同 Android）
- 不实现 R91 的原生热键（JS 路径已存在，仅需验证）

## Decisions

### D1: R77 set_focus — Float 子窗口通过 WindowManager 聚焦

**决策**: `tao` OHOS `set_focus` 调用 `openharmony-ability` 的窗口聚焦 API。主窗口（UIAbility）聚焦由 OS 管理（`onActive`/`onForeground`），`set_focus` 对主窗口为 no-op（同 iOS/Android）。

**实现路径**: `tao` → `openharmony-ability::window` → ArkTS `WindowManager` → `window.raiseToTop()` + `window.setWindowFocusable(true)`

**set_focusable**: 设置窗口是否可聚焦。OHOS `window.setWindowFocusable(isFocusable: boolean)` 支持。Float 子窗口可调用；主窗口由 OS 管理为 no-op。

**替代方案**: 直接在 tao 中调用 NAPI → 不可行，违反铁律 1（openharmony-ability 是唯一 ArkTS 桥接）。

### D2: R75 with_https_scheme — 添加 OHOS 平台特有属性

**决策**: 在 `PlatformSpecificWebViewAttributes` OHOS 变体中添加 `use_https: bool` 字段。`WebViewBuilderExtOhos` 新增 `with_https_scheme(self, enabled: bool) -> Self`。当 `enabled = true` 时，custom protocol 注册为 `https://` 前缀（满足 secure-context 要求）。

**ArkTS 侧**: custom protocol handler 已通过 `CustomProtocolHandler` 注册，scheme 由 wry 层决定。OHOS ArkWeb 的 `custom_protocol_async` 接受任意 scheme 字符串，因此只需在 wry 层将 protocol name 从 `tauri` 改为 `https` 或在 ArkTS 侧映射。

**与 Windows/Android 对比**: Windows 的 `with_https_scheme` 将 `http://tauri.localhost` 映射为 `https://tauri.localhost`。OHOS 同理。

### D3: R82 剪贴板 — 标注"始终启用"，属性无效

**决策**: OHOS ArkWeb 默认允许页面剪贴板访问（`document.execCommand('copy'/'cut'/'paste')`、Clipboard API）。wry 的 `clipboard` 属性是 Linux/Windows 专用（控制 WebKitGTK/WebView2 的剪贴板权限），OHOS 无对应 toggle。

**行为**: `.with_clipboard(true/false)` 在 OHOS 上被静默忽略，剪贴板始终可用。同 macOS 行为。

### D4: R85 数据存储标识 — 设计决策，非缺口

**决策**: OHOS Web 组件自动使用应用沙箱目录存储 web 数据（cookies、localStorage、cache）。`data_directory` 在 `tauri/manager/webview.rs:537-560` 中被 `cfg` 排除（同 Android）。这是设计决策，非缺口。

### D5: R86 数据目录 — 已实现，核实双重 join

**决策**: `PathResolver` OHOS 实现已完整。需核实 `base_path`（来自 `context.filesDir`，值形如 `/data/storage/el2/base/.../files`）与 `PathResolver::app_data_dir()` 的 `base_path.join("files")` 是否产生双重 `files/files` 路径。

**修复**: 如果确认双重 join，应将 `base_path` 改为 `context` 的 el2 base（不包含 `files` 后缀），或移除 `join("files")`。

### D6: R90 点击穿透 — 平台限制

**决策**: OHOS 无窗口级 `setIgnoreMouseEvents` API。ArkUI 的 `.hitTestBehavior(HitTestMode.None)` 是组件级，不是窗口级。`tao` OHOS 实现返回 `NotSupported`（与 iOS/Android 一致）。标注为平台限制。

### D7: R91 热键缩放 — 已实现，验证桌面端

**决策**: JS 路径（`zoom-hotkey.js`）已通过 `cfg(all(desktop, not(target_os = "windows")))` 注入到 OHOS 桌面端。程序缩放（`controller.zoom`）已实现。`set_webview_zoom` IPC 命令已注册（`cfg(desktop)`）。

**验证项**: OHOS 桌面端 ArkWeb 是否正确派发 `keydown` 事件（含 `ctrlKey`）和 `wheel` 事件。如果 ArkWeb 不派发这些事件，热键缩放不会触发。

**移动端**: 正确不激活（无 `cfg(desktop)`）。

## Risks / Trade-offs

- **[Risk] R77 Float 窗口聚焦 API** → 需确认 `openharmony-ability::WindowManager` 是否有 `raiseToTop` 或类似方法。如果没有，需要在 ArkTS 侧新增。→ 缓解：探查 `WindowManager.ets` 现有方法。
- **[Risk] R75 HTTPS scheme 注册** → OHOS ArkWeb 的 custom protocol 注册是否接受 `https` 作为 scheme 名需验证。如果 ArkWeb 限制 `https` 为保留 scheme，需要使用其他方式。→ 缓解：先测试 `custom_protocol_async("https", ...)` 是否工作。
- **[Risk] R86 双重 join** → 如果 `context.filesDir` 已包含 `/files` 后缀，`join("files")` 会产生 `/files/files`。→ 缓解：在设备上打印 `base_path` 值确认。
- **[Trade-off] R77 主窗口 set_focus 为 no-op** → 与 iOS/Android 一致，主窗口聚焦由 OS 管理。桌面端用户可能期望 `set_focus` 对主窗口生效，但 OHOS UIAbility 的聚焦由系统调度。
- **[Trade-off] R90 不实现** → 点击穿透是桌面 overlay 窗口功能。OHOS 无 API，标注为平台限制。未来如果 OHOS 增加窗口级 API，可再实现。
