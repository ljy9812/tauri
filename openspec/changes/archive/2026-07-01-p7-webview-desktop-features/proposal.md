## Why

Phase 7 覆盖 7 项桌面系 WebView 功能（R75/R77/R82/R85/R86/R90/R91）。探查发现：3 项已实现或已处理（R85/R86/R91），2 项是平台限制需标注（R82/R90），2 项需实现或增强（R75/R77）。本 Phase 的核心工作是补齐 stub 实现、标注平台限制、核实已实现项的行为一致性。

## What Changes

### 需要实现或增强（2 项）

- **R77 窗口聚焦** — `tao/src/platform_impl/ohos/mod.rs:813-820` 的 `set_focus` 和 `set_focusable` 是 FIXME stub（`warn!` + 无操作）。通过 `openharmony-ability` 的 `WindowManager` 调用 OHOS 窗口 API 实现 Float 子窗口聚焦；主窗口（UIAbility）聚焦由 OS 生命周期管理，标注为不适用。
- **R75 HTTPS 自定义协议** — OHOS 当前只支持 raw scheme（`tauri://localhost`）。添加 `WebViewBuilderExtOhos::with_https_scheme(bool)` 使 OHOS 支持 HTTPS origin 语义（`https://tauri.localhost`），用于满足 secure-context 要求。

### 平台限制标注（2 项）

- **R82 剪贴板（webview 属性）** — wry 的 `clipboard` 属性是 Linux/Windows 专用。OHOS ArkWeb 默认允许页面剪贴板访问（同 macOS），属性被静默忽略。标注为"始终启用，属性无效"。
- **R90 点击穿透** — OHOS 无窗口级 `setIgnoreMouseEvents` API。`tao` OHOS 实现返回 `NotSupported`。标注为平台限制，移动端不适用，桌面端未来可能通过窗口管理器扩展。

### 已实现/已处理（3 项，仅需核实或文档化）

- **R85 数据存储标识** — `tauri/manager/webview.rs:537-560` 有意不使用 `data_directory`（同 Android），OHOS Web 组件自动使用应用沙箱目录。标注为"设计决策，非缺口"。
- **R86 数据目录** — `tauri/path/ohos.rs` 已完全实现 PathResolver。需核实 `base_path`（`context.filesDir`）与 `join("files")` 的双重拼接问题。
- **R91 热键缩放** — JS 路径（`zoom-hotkey.js`）已通过 `cfg(desktop)` 注入，程序缩放（`controller.zoom`）已实现。需验证 OHOS 桌面端 ArkWeb 是否正确派发 `keydown` + `ctrlKey` 事件。

## Capabilities

### New Capabilities
- `webview-window-focus`: OHOS Float 子窗口的 `set_focus` / `set_focusable` 实现
- `webview-https-scheme`: OHOS WebView 的 HTTPS 自定义协议 toggle

### Modified Capabilities
None.

## Impact

- **2 repos modified**: tao（`set_focus`/`set_focusable` 实现），wry（`with_https_scheme` + `WebViewBuilderExtOhos`）
- **openharmony-ability**: 可能需要新增窗口聚焦 NAPI 方法（如果 `WindowManager` 没有现成的 focus API）
- **tao**: `set_focus` 从 `warn!` stub 改为调用 ability 窗口 API；`set_focusable` 同理
- **wry**: `PlatformSpecificWebViewAttributes` OHOS 变体增加 `use_https` 字段；`WebViewBuilderExtOhos` 增加 `with_https_scheme` 方法
- **不影响桌面平台**：所有修改通过 `cfg(target_env = "ohos")` 隔离
- **R82/R85/R86/R90/R91** 不需要代码修改，仅在 spec 中标注状态
