## 1. tauri crate — OHOS NewWindowResponse 枚举添加 Create 变体（审计新增）

- [x] 1.1 在 `crates/tauri/src/webview/mod.rs:268-274` 的 OHOS `NewWindowResponse` 枚举中添加 `Create { window: crate::WebviewWindow<R> }` 变体
- [x] 1.2 在 `crates/tauri/src/webview/mod.rs:725-739` 的 match 站点添加 `#[cfg(target_env = "ohos")]` `Create { window }` arm，映射到 `tauri_runtime::webview::NewWindowResponse::Create { window_id: window.window.id() }`（需确认 OHOS 上 `WebviewWindow` 的 window ID 获取方式）
- [x] 1.3 更新 `examples/api/src-tauri/src/lib.rs` 的 `on_new_window_req` handler，在 OHOS 分支中支持返回 `Create`（当前 OHOS 分支返回 `Deny`）
- [x] 1.4 `cargo check` 验证 tauri crate 在 OHOS target 编译通过

## 2. tauri-runtime — 解除 Create 变体的 OHOS 排除

- [x] 2.1 在 `crates/tauri-runtime/src/webview.rs` 中，从 `NewWindowResponse::Create` 的 `cfg` gate 移除 `target_env = "ohos"`（line 174）
- [x] 2.2 从 `WindowId` import 的 `cfg` gate 移除 `target_env = "ohos"`（line 8）
- [x] 2.3 全局搜索 `match.*NewWindowResponse` 确认所有 match 站点在 OHOS 上能处理 `Create` 变体（非穷尽检查）
- [x] 2.4 `cargo check` 验证 tauri-runtime 在 OHOS target 编译通过

## 3. tauri-runtime-wry — 添加 OHOS Create arm

- [x] 3.1 在 `crates/tauri-runtime-wry/src/lib.rs` 中，为 `Create` match arm 添加 `#[cfg(target_env = "ohos")]` 分支，构造 `wry::NewWindowResponse::Create { }`（无字段）
- [x] 3.2 确保桌面平台的 `Create` arm（webview lookup）不受影响（`#[cfg(all(desktop, not(target_env = "ohos")))]` 保持不变）
- [x] 3.3 `cargo check` 验证 tauri-runtime-wry 在 OHOS target 编译通过

## 4. openharmony-ability Rust — 扩展 OnWindowNewResult + handler 返回类型

- [x] 4.1 在 `crates/ability/src/helper/webview.rs` 中，为 `OnWindowNewResult` 添加 `pub window_kind: Option<String>` 字段
- [x] 4.2 更新 `OnWindowNewResult` 的 `Default` impl，`window_kind` 默认为 `None`
- [x] 4.3 在 `crates/ability/src/webview/mod.rs` 中，将 `on_window_new` builder 方法的泛型约束从 `F: Fn(String, bool, bool) -> bool` 改为 `F: Fn(String, bool, bool) -> OnWindowNewResult`
- [x] 4.4 更新 NAPI `create_function_from_closure` 闭包，直接返回 handler 的 `OnWindowNewResult`（不再包装 `bool`）
- [x] 4.5 新增 `#[napi]` 函数 `generate_window_id() -> i64`，复用 `NEXT_WINDOW_ID` 原子计数器
- [x] 4.6 `cargo check` 验证 openharmony-ability 编译通过

## 5. wry OHOS — Create 返回 window_kind

- [x] 5.1 在 `wry/src/ohos/mod.rs` 中，将 `on_window_new` 桥接闭包的返回类型从 `bool` 改为 `OnWindowNewResult`
- [x] 5.2 `Allow` → `OnWindowNewResult { allow: true, window_kind: None }`
- [x] 5.3 `Create { .. }` → `OnWindowNewResult { allow: true, window_kind: Some("window".to_string()) }`（替换现有的 `log::warn!` 降级代码）
- [x] 5.4 `Deny` → `OnWindowNewResult { allow: false, window_kind: None }`
- [x] 5.5 `cargo check` 验证 wry OHOS 编译通过

## 6. openharmony-ability ArkTS — handleWindowNew 路由真窗口

- [x] 6.1 在 `native_ability/src/main/ets/ability/type.ets` 中，为 `OnWindowNewResult` 接口添加 `window_kind?: string` 字段
- [x] 6.2 在 `native_ability/src/main/ets/webview/DefaultWebview.ets` 的 `handleWindowNew` 中，当 `result.window_kind === 'window'` 时：
  - [x] 6.2a 同步调用 `event.handler.setWebController(newCtrl)`（ArkWeb 合约）
  - [x] 6.2b `setTimeout(() => { ... }, 0)` 延迟执行
  - [x] 6.2c 在延迟回调中调用 `generateWindowId()` 获取窗口 ID
  - [x] 6.2d 调用 `WindowManager.createSubWindow({ name, windowId, ... })` 创建真窗口
  - [x] 6.2e 调用 `WindowManager.loadUrl(windowId, targetUrl)` 加载 URL
- [x] 6.3 确保 `result.window_kind` 为 `None`/`undefined`/`"dialog"` 时走现有 `openNewWindowDialog` 路径（不变）
- [x] 6.4 重建 HAR：`ohrs build --arch arm64 -p openharmony-ability` + `cp -r native_ability/src/main/ets package/src/main/ets` + `tar -czf ability.har package`

## 7. 构建部署验证

- [x] 7.1 OHOS desktop `cargo check` 通过（3-env 验证：Windows host + OHOS desktop）
- [x] 7.2 构建部署到设备：`run-tests.sh "" desktop`
- [x] 7.3 手动测试：在 TestRunner 中调用 `window.open()`，验证 `Create` 打开真窗口、`Allow` 打开对话框、`Deny` 阻止
- [x] 7.4 验证 `Allow`/`Deny` 路径不受影响（回归测试）

## 8. 更新 plan 文件

- [x] 8.1 更新 `openspec/webview-gap-completion-plan.md` 中 Phase 6 状态为 `✓ 设计完成`
