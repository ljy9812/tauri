## 1. ArkTS 桥接层（openharmony-ability）

- [x] 1.1 `native_ability/src/main/ets/webview/Utils.ets`：新增模块级 `let webDebuggingEnabled = false;`；新增 `setWebDebuggingAccess(enabled: boolean)` 常量（**先 `web_webview.WebviewController.setWebDebuggingAccess(enabled);` 再 `webDebuggingEnabled = enabled;`**——API 抛异常则变量不更新，保持准确）与 `isWebDebuggingAccess(): boolean` 常量（`return webDebuggingEnabled;`）。状态变量与常量必须同文件
- [x] 1.2 `native_ability/src/main/ets/webview/Utils.ets`：JsHelper 接口新增 `setWebDebuggingAccess: (enabled: boolean) => void` 与 `isWebDebuggingAccess: () => boolean`；ProxyJsHelper 实现——`setWebDebuggingAccess` 直接调 Utils 常量（静态 API，无需控制器，**不走 pendingOperations**）、`isWebDebuggingAccess` 直接返回 `webDebuggingEnabled` 模块变量（不返回硬编码 false）
- [x] 1.3 `native_ability/src/main/ets/webview/DefaultWebview.ets`：import 补充 `setWebDebuggingAccess, isWebDebuggingAccess`；buildJsHelper 接入两 helper 加入返回对象；init 处（`:384`）既有 `web_webview.WebviewController.setWebDebuggingAccess(true)` 改为调 `setWebDebuggingAccess(true)` 常量（同步状态变量）
- [x] 1.4 重建 HAR 包（`ohrs build --arch arm64` + 刷新 `package/src/main/ets` + 重打包 `ability.har`）— HAR 已含 setWebDebuggingAccess×7/isWebDebuggingAccess×3/webDebuggingEnabled×4（Utils.ets）

## 2. Rust NAPI 层（openharmony-ability）

- [x] 2.1 `crates/ability/src/helper/webview.rs`：新增 `Webview::set_web_debugging_access(&self, enabled: bool) -> Result<()>`，经 `get_named_property::<Function<'_, bool, ()>>("setWebDebuggingAccess")` 调用（参考 `setVisible`）
- [x] 2.2 `crates/ability/src/helper/webview.rs`：新增 `Webview::is_web_debugging_access(&self) -> Result<bool>`，经 `get_named_property::<Function<'_, (), bool>>("isWebDebuggingAccess")` 调用（参考 `url` 的 `Function<'_, (), String>`）

## 3. wry OHOS 层

- [x] 3.1 `wry/src/ohos/mod.rs`：`open_devtools`（返回 `()`）改为 `if let Err(e) = self.webview.set_web_debugging_access(true) { log::warn!("[wry] open_devtools failed: {}", e); }`（保留 `#[cfg(any(debug_assertions, feature = "devtools"))]`）
- [x] 3.2 `wry/src/ohos/mod.rs`：`close_devtools`（返回 `()`）改为 `if let Err(e) = self.webview.set_web_debugging_access(false) { log::warn!("[wry] close_devtools failed: {}", e); }`（保留 cfg 门控）
- [x] 3.3 `wry/src/ohos/mod.rs`：`is_devtools_open` 改为 `self.webview.is_web_debugging_access().unwrap_or(false)`（保留 cfg 门控）
- [x] 3.4 `wry/src/ohos/mod.rs`：`focus_parent` 改为 `self.webview.focus().map_err(|e| Error::OpenHarmonyWebviewError(format!("Failed to focus parent: {}", e)))`（无 cfg 门控；加注释说明 OHOS 无独立父窗口，focus webview 即等价）
- [x] 3.5 加注释标注 `setWebDebuggingAccess` 为进程级全局生效（区别于桌面 per-webview devtools）— 已在 open_devtools 注释块 + focus_parent 注释

## 4. 编译验证

- [x] 4.1 `cargo check --target aarch64-unknown-linux-ohos -p openharmony-ability` 通过（`Function<'_, (), bool>` 返回类型验证可用）
- [x] 4.2 `cargo check --target aarch64-unknown-linux-ohos --features openharmony-ability/webview`（wry）通过（仅既有 warning）
- [x] 4.3 `cargo check`（host 非 ohos）通过，未影响其他平台
- [x] 4.4 设备冒烟（无回归）：刷新 ability.har → 构建+部署 api demo → cookie 自动用例 4/4 通过，全套 208/210（2 个既有无关失败：RunEvent::Resumed、clipboard writeText），无新增失败

## 5. 设备端验证

- [x] 5.1 **测试构建准备**（已完成，验证后已回退）：在 api demo 临时启用 `devtools = ["tauri/devtools"]` feature 并加入 `prod`（仅验证用），使 devtools 三方法在 release 构建编译；新增 `devtools_test` 命令（function-level `#[cfg(any(debug_assertions, feature="devtools"))]` 门控）+ ACL + 前端按钮/自动用例作为测试钩子
- [x] 5.2 启用 devtools feature 的构建部署后验证（设备 3QC0124C11000845，desktop）：`devtools_test` 自动用例通过——`open_devtools()` 后 `is_devtools_open()=true`、`close_devtools()` 后 `=false`。注：`initial=true` 为正确行为（tauri `WebviewWindowBuilder` 默认 `devtools=true`，init 标志置位）
- [ ] 5.3 `focus_parent()` 调用后 webview 获得焦点（manual，release 可测）— **不可行**：focus_parent 为 dead public API（无外部调用方，审计已确认），api demo 无法触发
- [x] 5.4 标准 release 构建（无 devtools feature）确认 devtools 三方法不编译（符合 cfg 门控，与其他平台一致）— 4.4 冒烟构建即 release，devtools 方法未编译，app 正常运行

## 6. 文档与标注

- [x] 6.1 wry 注释标注：`is_devtools_open` 返回自跟踪状态（OHOS 无 getter）；`setWebDebuggingAccess` 进程全局；`focus_parent` 聚焦 webview 本身（无独立父窗口）— 已在 mod.rs 注释
- [x] 6.2 确认改动仅位于 `cfg(target_env="ohos")` 路径（devtools 三方法保留既有 cfg 门控），未影响其他平台 — host cargo check 通过
