## Why

wry OHOS 的 `set_bounds` 对**非子 webview**（主 webview）是 cache-only（仅更新 `bounds_cache`，不调 ArkTS `setBounds`），导致主 webview 的 `set_bounds()` 调用无实际效果——bounds 缓存更新了但 Web 组件未重渲染。直接移除 cache-only 会导致全屏黑边（Web 组件 `"100%"` 被替换为具体像素值后，窗口 resize 时 set_bounds 未被调用）。根因是 tao 不传播 `ContentRectChange` 为 `Resized` 事件 + `WindowIdStore` 的 ZST key 被子窗口覆盖。R74 透明背景经 archive `p1-webview-transparent` 已实现（仅子窗口，主窗口窗口级透明未实现）。

## What Changes

- **tao ContentRectChange 传播**（`tao/src/platform_impl/ohos/mod.rs`）：将 `ContentRectChange` 事件传播为 `WindowEvent::Resized(PhysicalSize)`（原为 TODO warn），使 tauri 的 resize handler 在窗口 resize/全屏时触发 → `set_bounds(新尺寸)` 被调用。
- **tauri-runtime-wry WindowIdStore or_insert**（`crates/tauri-runtime-wry/src/lib.rs`）：`insert` 改为 `entry(w).or_insert(id)`。OHOS 的 `WindowId` 是 ZST——所有窗口共享同一 HashMap key，子窗口创建会覆盖主窗口映射。`or_insert` 保留首个（主窗口）映射，防止子窗口覆盖。
- **wry set_bounds 移除 cache-only**（`wry/src/ohos/mod.rs`）：移除 `if !self.is_child { cache-only; return; }` 早返回。子与非子 webview 统一调 `self.webview.set_bounds(x, y, w, h)` + 更新缓存。前提是 tao 传播 resize 事件 + or_insert 确保 window_id 映射正确。
- **R74 透明背景**：核实 archive `p1-webview-transparent` 已落地（ArkHelper `init.transparent` + DefaultWebview `RenderMode.SYNC_RENDER` + 容器防御性透明 + `set_background_color` 动态更新）。**仅子窗口生效**（FloatPage 独立悬浮窗可透明），主窗口窗口级透明未实现 → R74 维持 ⚠️。

## Capabilities

### New Capabilities
- `webview-bounds-nonchild`: 非子（主）webview 的 `set_bounds` 调用 ArkTS `setBounds` 实际生效（经 `updateWebviewStyle` 重渲染 Web 组件）；窗口 resize/全屏时自动更新

### Modified Capabilities
- `webview-transparent-bg`（archive p1-webview-transparent）：R74 核实已落地，但仅子窗口透明，主窗口窗口级透明未实现 → 维持 ⚠️

## Impact

- **tao**（Rust）：`src/platform_impl/ohos/mod.rs` 的 `ContentRectChange` handler 从 warn TODO 改为传播 Resized（+45/-19）
- **tauri**（Rust）：`crates/tauri-runtime-wry/src/lib.rs` 的 `WindowIdStore::insert` 改为 `or_insert`（+4/-1）
- **wry**（Rust）：`src/ohos/mod.rs` 的 `set_bounds` 移除非子 cache-only 早返回（+8/-9）
- **tauri**（测试）：`examples/api` 新增 `set_bounds_test` 命令 + 自动用例（core.ts test 53）+ 手动用例更新（manual_tests.md 7.4）
- **openharmony-ability**：无改动（ArkTS `setBounds` 已实现）
- **平台一致性**：与 Windows/macOS 的 `set_bounds` 行为对齐（主 webview 可设 bounds + resize 自动更新）
- **铁律遵守**：wry/tao 改动限于 `cfg(target_env="ohos")` 路径；tauri-runtime-wry 改动限于 `or_insert` 一行（跨平台但仅影响 ZST key 行为）
