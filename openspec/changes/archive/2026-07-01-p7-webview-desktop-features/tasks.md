## 1. R77 — tao set_focus / set_focusable 实现

- [x] 1.1 探查 `openharmony-ability` `WindowManager.ets` 是否有 `raiseToTop` / `setWindowFocusable` 或类似方法
- [x] 1.2 如果没有，在 `openharmony-ability` 新增 `focus_window(window_id: i64) -> Result<()>` NAPI 方法（ArkTS 调 `window.raiseToTop()` + `window.setWindowFocusable(true)`）
- [x] 1.3 在 `tao/src/platform_impl/ohos/mod.rs:813` 实现 `set_focus`：Float 子窗口调 `focus_window`，主窗口 no-op
- [x] 1.4 在 `tao/src/platform_impl/ohos/mod.rs:818` 实现 `set_focusable`：Float 子窗口调 `set_window_focusable`，主窗口 no-op
- [x] 1.5 `cargo check` 验证 tao OHOS 编译通过

## 2. R75 — wry with_https_scheme

- [x] 2.1 在 `wry/src/lib.rs` `PlatformSpecificWebViewAttributes` OHOS 变体中添加 `use_https: bool` 字段
- [x] 2.2 在 `WebViewBuilderExtOhos` trait 中添加 `with_https_scheme(self, enabled: bool) -> Self` 方法
- [x] 2.3 在 `wry/src/ohos/mod.rs` `new_inner` 中读取 `use_https`，当 `true` 时将 custom protocol scheme 改为 `https`
- [x] 2.4 验证 OHOS ArkWeb 是否接受 `https` 作为 custom protocol scheme（可能需要 ArkTS 侧适配）
- [x] 2.5 `cargo check` 验证 wry OHOS 编译通过

## 3. R86 — 核实 PathResolver 双重 join

- [x] 3.1 在设备上打印 `base_path` 值，确认是否包含 `/files` 后缀
- [x] 3.2 如果确认双重 join（`/files/files`），修复 `tauri/crates/tauri/src/path/ohos.rs` 中的路径拼接
- [x] 3.3 如果 `base_path` 不含 `/files`（即 el2 base root），则当前实现正确，无需修改

## 4. R91 — 验证热键缩放（仅验证，不写代码）

- [x] 4.1 在 OHOS 桌面端打开 TestRunner，按 Ctrl+`=` 测试缩放
- [x] 4.2 检查 hilog 确认 `keydown` 事件是否被 ArkWeb 正确派发（含 `ctrlKey`）
- [x] 4.3 如果热键不工作，记录原因（ArkWeb 不派发 `keydown` / `ctrlKey` 未设置等）
- [x] 4.4 如果热键工作，记录为已验证

## 5. 平台限制标注（仅文档，不写代码）

- [x] 5.1 R82 剪贴板：确认 ArkWeb 默认允许剪贴板访问（在 TestRunner 中测试 `document.execCommand('copy')`）
- [x] 5.2 R85 数据存储标识：确认 `data_directory` 在 OHOS 上被忽略（检查 `manager/webview.rs` cfg gate）
- [x] 5.3 R90 点击穿透：确认 `set_ignore_cursor_events` 返回 `NotSupported`（检查 tao 代码）

## 6. 构建部署验证

- [x] 6.1 OHOS desktop `cargo check` 通过
- [x] 6.2 构建部署到设备：`run-tests.sh "" desktop`
- [x] 6.3 手动测试 R77 窗口聚焦（Float 子窗口 set_focus 后窗口前置）
- [x] 6.4 手动测试 R75 HTTPS scheme（如果实现了）
- [x] 6.5 验证 R82/R85/R90 行为符合 spec 标注

## 7. 更新 plan 文件

- [x] 7.1 更新 `openspec/webview-gap-completion-plan.md` 中 Phase 7 状态为 `✓ 设计完成`
- [x] 7.2 更新 Phase 3 状态为 `✓ 已归档`（如尚未更新）
- [x] 7.3 更新 Phase 4 状态为 `✗ 平台限制`（如尚未更新）
