## Context

wry OHOS `InnerWebView::set_bounds`（`wry/src/ohos/mod.rs`）：
- 非子（主）webview：`if !self.is_child { cache-only; return; }`（484-488）——仅更新 `bounds_cache`，不调 ArkTS
- 子 webview：调 `self.webview.set_bounds(x,y,w,h)` + 缓存（489-496）

直接移除 cache-only 会导致**全屏黑边**：set_bounds 在 app 启动时被调一次（设具体像素值替换 `"100%"`），但全屏时 set_bounds 未再被调用 → Web 组件停留在初始尺寸 → 黑边。

根因（3 点）：
1. **tao 不传播 `ContentRectChange`**：`tao/src/platform_impl/ohos/mod.rs:339` 仅 `warn!("TODO: find a way to notify application of content rect change")`，不传播为 `Resized` 事件 → tauri resize handler 不触发 → set_bounds 不被调用
2. **`WindowIdStore` ZST key 覆盖**：OHOS 的 `WindowId` 是 ZST（所有窗口共享同一 HashMap key），子窗口创建 `insert` 覆盖主窗口映射 → resize 事件映射到子窗口（WindowId(7)）而非主窗口（WindowId(0)）→ 主窗口的 webview 的 set_bounds 不被调用
3. **wry cache-only**：非子 webview set_bounds 不调 ArkTS setBounds → Web 组件 `"100%"` 保持不变（cache-only 是前两点的 workaround）

ArkTS `setBounds`（ArkHelper.ets:317-319 monkey-patch）→ `applyStyle({x,y,width,height})` → `updateWebviewStyle` → `node.update(newEntry)` → Web 组件按 `data.style.width/height/position` 重渲染。主 webview 的 Web 组件 `.width(data.style?.width ?? "100%")` + `.position({...})`（DefaultWebview.ets:119-121）——由 `data.style` 驱动布局，setBounds 生效。

R74 透明背景：archive `p1-webview-transparent` 已实现（ArkHelper `init.transparent=true`、DefaultWebview `RenderMode.SYNC_RENDER`、容器防御性透明、`set_background_color` 动态更新）。**仅子窗口**（FloatPage 独立悬浮窗）透明生效；主窗口窗口级透明（OHOS window API）未实现 → R74 维持 ⚠️。

## Goals / Non-Goals

**Goals:**
- R78：非子 webview `set_bounds` 真正生效（移除 cache-only），前提是修复 tao resize 传播 + or_insert 防覆盖
- R74：核实透明背景，标注仅子窗口生效

**Non-Goals:**
- 不实现主窗口窗口级透明（需 OHOS window API，超出本 Phase 范围）
- 不改动 ArkTS `setBounds`（已实现）

## Decisions

### D1: tao 传播 ContentRectChange 为 Resized
`tao/src/platform_impl/ohos/mod.rs` 的 `ContentRectChange` handler 从 `warn!("TODO:...")` 改为传播 `WindowEvent::Resized(PhysicalSize::new(rect.width, rect.height))`。`ContentRect` 携带 `rect: Rect { width, height }`（physical px），直接构造 `PhysicalSize`。与 `WindowResize` handler（321-331）模式一致。

### D2: WindowIdStore or_insert
`tauri-runtime-wry/src/lib.rs` 的 `WindowIdStore::insert` 改为 `entry(w).or_insert(id)`。OHOS 的 `WindowId` 是 ZST（`#[derive(Hash, PartialEq, ...)] struct WindowId;`），所有窗口实例 hash 相等 → HashMap 仅一个 entry。`insert` 会覆盖，`or_insert` 保留首个（主窗口）。子窗口事件仍能到达（OHOS 单窗口模型，所有事件走同一 event loop handler）。

### D3: wry set_bounds 移除 cache-only
移除 `if !self.is_child { cache-only; return; }` 早返回。子与非子统一调 `self.webview.set_bounds(x,y,w,h)` + 更新 `bounds_cache`。前提：D1（resize 传播）+ D2（or_insert 防覆盖）确保 set_bounds 在每次 resize 时被正确调用 → Web 组件更新到新尺寸 → 无黑边。

### D4: R74 透明背景维持 ⚠️
archive `p1-webview-transparent` 全部改动已在代码中。**仅子窗口**（FloatPage 独立悬浮窗）透明生效；主窗口窗口级透明（需 OHOS `window.setWindowBackgroundColor` 等 API）未实现 → R74 维持 ⚠️。

## Risks / Trade-offs

- **or_insert 影响子窗口事件**：`or_insert` 保留首个（主窗口）映射，子窗口的 tao 事件也映射到主窗口的 tauri WindowId。但 OHOS 是单窗口模型（所有事件走同一 event loop handler），子窗口事件由 handler 内部按 window_id 分发——`or_insert` 不影响子窗口的 `windows.0.borrow().get(&window_id)` 查找（子窗口的 tauri WindowId 是独立的，由 `next_window_id` 分配，不依赖 `window_id_map`）。→ 接受；设备验证无回归。
- **ContentRectChange 频率**：`windowRectChange` 可能在安全区域变化等场景频繁触发 → 多余的 Resized 事件 → set_bounds 被多次调用。但 set_bounds 是幂等的（设同样的值无副作用），且 `updateWebviewStyle` → `node.update` 是轻量重渲染。→ 接受。
- **主窗口透明未实现**：R74 维持 ⚠️（仅子窗口）。主窗口透明需 OHOS window API（`setWindowBackgroundColor` + `setWindowMode`），属后续 Phase。→ 接受。
