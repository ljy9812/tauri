## Context

wry 在 OHOS 上的 `InnerWebView`（`wry/src/ohos/mod.rs`）：

- `open_devtools()`/`close_devtools()` → 空 `{}`（`#[cfg(any(debug_assertions, feature = "devtools"))]` 门控）
- `is_devtools_open()` → 硬编码 `false`
- `focus_parent()` → `Ok(())` no-op

`openharmony-ability::helper::webview::Webview` 已有 `focus()`（`helper/webview.rs:254`）→ ArkTS `requestFocus` NAPI，可复用。

ArkTS 侧 `DefaultWebview.ets:384` 在 init 时若 `devtools` 标志为真则一次性调 `web_webview.WebviewController.setWebDebuggingAccess(true)`，但无运行时 toggle、无状态查询。

OHOS `WebviewController.setWebDebuggingAccess(bool)`（`@kit.ArkWeb`，arkts-helper 文档核实）：
- **静态方法**，全局生效（对所有 Web 组件），默认 false
- **无 getter**（`isWebDebuggingAccess` 不存在）
- 须在 UI 线程调用

结论：`is_devtools_open` 无法查询 OHOS 实际状态，须在 ArkTS 侧自维护 `webDebuggingEnabled` 变量；`focus_parent` 复用 `Webview::focus()`（OHOS 无独立父窗口，聚焦 webview 即等价）。

## Goals / Non-Goals

**Goals:**
- `open_devtools`/`close_devtools` 端到端可用（wry → ability NAPI → ArkTS `setWebDebuggingAccess` + 状态更新）
- `is_devtools_open` 返回 ArkTS 侧维护的真实开关状态（含 init 初始值与运行时 toggle）
- `focus_parent` 复用 `Webview::focus()`（requestFocus）
- 遵守铁律：ArkTS 调用仅经 openharmony-ability；wry 改动限于 OHOS cfg 路径；静态方法用 `get_main_thread_env()`

**Non-Goals:**
- 不实现"独立 DevTools 窗口"（移动端/OHOS 无此能力；桌面平台靠 OS 自带。本 Phase 仅切换 `setWebDebuggingAccess` 调试访问开关）
- 不查询 OHOS 实际 debugging 状态（无 getter；返回自跟踪值，文档标注）
- 不改动 devtools 三方法的 `#[cfg(any(debug_assertions, feature = "devtools"))]` 门控（与 webview2/wkwebview 一致）
- 不改动 init 时 `setWebDebuggingAccess(true)` 的既有调用位置（`:384` 的 `if (prepared?.devtools)` 块保留；仅将该行的直接静态 API 调用改为调 Utils 常量，以同步状态变量）

## Decisions

### D1: 状态跟踪放在 ArkTS 侧（Utils.ets，与常量同文件）
因 `setWebDebuggingAccess` 无 getter，且 init 时需在 ArkTS 调用它，状态跟踪放 ArkTS 最内聚。状态变量与 `setWebDebuggingAccess`/`isWebDebuggingAccess` 常量**必须同文件**（Utils.ets），否则常量无法访问变量：
- `Utils.ets` 模块级 `let webDebuggingEnabled = false;`
- `Utils.ets` 常量 `setWebDebuggingAccess(enabled)`：`web_webview.WebviewController.setWebDebuggingAccess(enabled); webDebuggingEnabled = enabled;`（**先调 API 再更变量**——若 API 抛异常则变量不更新，保持状态准确）
- `Utils.ets` 常量 `isWebDebuggingAccess()`：`return webDebuggingEnabled;`
- `DefaultWebview.ets` init 处（`:384`）既有 `web_webview.WebviewController.setWebDebuggingAccess(true)` 改为调用 `Utils.ets` 的 `setWebDebuggingAccess(true)` 常量（需 import），以同步状态变量

### D2: NAPI 桥接两个方法
- Rust `Webview::set_web_debugging_access(&self, enabled: bool) -> Result<()>` → JsHelper `setWebDebuggingAccess`（`FnArgs<(bool,)>`？无需——单参 bool，用 `Function<'_, bool, ()>`，参考 `zoom` 的 `Function<'_, f64, ()>` 模式）
- Rust `Webview::is_web_debugging_access(&self) -> Result<bool>` → JsHelper `isWebDebuggingAccess`（`Function<'_, (), bool>`，参考 `getUrl` 的 `Function<'_, (), String>` 模式）

### D3: wry 三方法映射
- `open_devtools`（返回 `()`）：`if let Err(e) = self.webview.set_web_debugging_access(true) { log::warn!("[wry] open_devtools failed: {}", e); }`（返回 `()` 与桌面契约一致；**记录告警**而非静默忽略，便于排查）
- `close_devtools`（返回 `()`）：同上，`set_web_debugging_access(false)` + `log::warn!` on error
- `is_devtools_open`（返回 `bool`）：`self.webview.is_web_debugging_access().unwrap_or(false)`
- 三方法保留 `#[cfg(any(debug_assertions, feature = "devtools"))]` 门控

### D4: focus_parent 复用 focus()（含错误映射）
- `focus_parent`（返回 `wry::Result<()>`）：`self.webview.focus().map_err(|e| Error::OpenHarmonyWebviewError(format!("Failed to focus parent: {}", e)))`（ability→wry Error 转换，参考 `zoom:327`）
- 无 cfg 门控
- **语义差异（文档标注）**：桌面平台 `focus_parent` 聚焦**父窗口**（webkitgtk `parent_window().focus()`、webview2 `SetFocus(parent HWND)`）；OHOS webview 无独立父窗口（webview 即窗口内容），故聚焦 webview 本身。对子 webview 场景这并非严格等价，但 **`focus_parent` 当前无外部调用方**（仅 wry/lib.rs:2234 委托，tauri/tao 无 `.focus_parent()` 调用，是 dead public API），实现为 wry 契约补全，影响为零

### D5: ProxyJsHelper 行为
- `setWebDebuggingAccess(enabled)`：`setWebDebuggingAccess` 是**静态全局 API**（无需控制器实例）→ ProxyJsHelper 直接调用 `Utils.ets` 常量（立即生效，**不走 pendingOperations 回放**，区别于 `setCookie`/`zoom` 等需控制器的动作）
- `isWebDebuggingAccess()`：直接返回 `Utils.ets` 的 `webDebuggingEnabled` 模块变量（即使控制器未绑定也能反映 init 时设置的状态，**不返回硬编码 false**）

## Risks / Trade-offs

- **`is_devtools_open` 返回自跟踪值非实查**：若外部直接调 `WebviewController.setWebDebuggingAccess` 绕过 wry，状态会不一致。→ 缓解：wry 是唯一调用方；spec 标注为"自跟踪状态"
- **`is_devtools_open` 跨平台语义差异**：webkitgtk 返回 inspector-window-open 状态（`is_inspector_open`），webview2 硬编码 `false`（stub），OHOS 返回 debugging-access-enabled 状态。三者语义不同（"devtools 窗口是否开" vs "调试访问是否启用"），但 wry 跨平台契约本就宽松（webview2 stub），OHOS 因无 devtools 窗口改返访问开关属合理近似；spec 标注
- **`setWebDebuggingAccess` 全局生效**：多 webview 场景下，一个 webview 的 open/close 影响所有。→ 接受（OHOS API 即如此；与桌面 per-webview devtools 行为不同，文档标注）
- **devtools 门控**：标准 OHOS 构建为 `--release --features prod`（`debug_assertions` 关、未启 `wry/devtools`），release 下三方法不编译，"支持"仅在 debug/devtools 构建生效。→ 接受（与 webview2/wkwebview 一致）；**设备验证需在测试构建中启用 `wry/devtools` feature**（`focus_parent` 无门控，release 可测）
- **focus_parent 语义差异**：桌面聚焦父窗口（HWND/GdkWindow），OHOS 聚焦 webview 本身（无独立父窗口）。→ 接受；**`focus_parent` 当前为 dead public API（无外部调用方）**，实现为 wry 契约补全，影响为零；spec 标注
- **安全性**：`setWebDebuggingAccess(true)` 有安全隐患（OHOS 文档：可检查修改 Web 内部状态，不建议正式发布启用）。→ 缓解：wry 仅在 `#[cfg(any(debug_assertions, feature="devtools"))]` 下传 `devtools=true`（mod.rs:67/136），release 构建不触发；ArkTS init `:384` 虽常驻但仅在 devtools 标志真时执行（仅 debug/devtools 构建）
- **状态持久性**：`webDebuggingEnabled` 为模块级变量，跨 webview 生命周期持久。→ 正确（`setWebDebuggingAccess` 本身是进程级粘性全局，一旦启用持续生效至显式关闭）；spec 标注
