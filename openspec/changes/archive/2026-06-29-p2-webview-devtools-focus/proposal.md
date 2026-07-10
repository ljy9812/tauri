## Why

wry OHOS WebView 的 DevTools 三方法为空/stub：`open_devtools`/`close_devtools` 是空 `{}`、`is_devtools_open` 硬编码返回 `false`；`focus_parent` 是 `Ok(())` no-op。开发者无法在 OHOS 上开启网页调试或查询其状态，也无法聚焦 webview，与 Windows/macOS 行为不一致。

经查证 OHOS `WebviewController.setWebDebuggingAccess(bool)`（`@kit.ArkWeb`）：是**静态全局** setter（默认 false，对所有 Web 组件生效），但**没有 getter**。因此 `is_devtools_open` 无法查询 OHOS 实际状态，需在 ArkTS 侧自维护开关状态。`focus_parent` 在桌面平台聚焦父窗口 HWND；OHOS 无独立父窗口（webview 即窗口内容），等价于聚焦 webview 本身——`openharmony-ability` 已有 `Webview::focus()`（requestFocus NAPI），可直接复用。

## What Changes

- **`open_devtools()`**：调 `WebviewController.setWebDebuggingAccess(true)` 并将 ArkTS 侧状态置 true。
- **`close_devtools()`**：调 `WebviewController.setWebDebuggingAccess(false)` 并将 ArkTS 侧状态置 false。
- **`is_devtools_open()`**：返回 ArkTS 侧维护的状态（非 OHOS 实查，因无 getter）。状态由 init 时的 devtools 标志与 open/close 调用共同维护。
- **`focus_parent()`**：复用 `openharmony-ability::helper::webview::Webview::focus()`（requestFocus）。标注平台差异：OHOS 无独立父窗口，聚焦 webview 本身。
- **ArkTS 状态维护**（Utils.ets，与常量同文件）：新增模块级 `webDebuggingEnabled` 变量；新增 `setWebDebuggingAccess(enabled)` 常量（更新变量 + 调静态 API）与 `isWebDebuggingAccess()` 常量（返回变量）；DefaultWebview.ets init 处既有 `setWebDebuggingAccess(true)` 改调 Utils 常量以同步状态。状态变量与常量必须同文件（否则常量无法访问变量）。
- **openharmony-ability NAPI**：`helper/webview.rs` 新增 `set_web_debugging_access(bool)` 与 `is_web_debugging_access() -> bool`，经 JsHelper 桥接。
- devtools 三方法保持既有 `#[cfg(any(debug_assertions, feature = "devtools"))]` 门控（与 webview2/wkwebview 一致）；`focus_parent` 无门控。

## Capabilities

### New Capabilities
- `webview-devtools-focus`: OHOS WebView 调试访问开关（open/close/is_devtools_open，ArkTS 侧状态跟踪）与父窗口聚焦（focus_parent → requestFocus）

### Modified Capabilities
（无）

## Impact

- **wry**（Rust）：`src/ohos/mod.rs` 的 `open_devtools`/`close_devtools`/`is_devtools_open`/`focus_parent` 替换 stub
- **openharmony-ability**（Rust）：`crates/ability/src/helper/webview.rs` 新增 `set_web_debugging_access`/`is_web_debugging_access` NAPI 方法
- **openharmony-ability**（ArkTS）：`native_ability/src/main/ets/webview/Utils.ets` 新增 `webDebuggingEnabled` 模块变量 + `setWebDebuggingAccess`/`isWebDebuggingAccess` 常量 + JsHelper 接口 + ProxyJsHelper（set 直接调常量、is 返回模块变量）；`DefaultWebview.ets` buildJsHelper 接入两 helper、init 处改调 Utils `setWebDebuggingAccess` 常量
- **平台一致性**：devtools 三方法与 Windows/macOS 的 cfg 门控一致；`is_devtools_open` 因 OHOS 无 getter 返回自跟踪状态（文档标注）
- **铁律遵守**：ArkTS 调用经 openharmony-ability；wry 改动限于 `cfg(target_env="ohos")`；`setWebDebuggingAccess` 为静态方法（UI 线程），复用 `get_main_thread_env()` 模式
