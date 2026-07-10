## Context

`PredefinedActionExecutor.execute()` 中 6 个剪贴板/编辑操作（copy/cut/paste/selectAll/undo/redo）使用 `this.controller`（`web_webview.WebviewController`）执行 JS。该 controller 通过 `setPrimaryWebviewControllerCallback` 在 NativeAbility 初始化时设置，永远是主窗口的 webview controller。

多窗口场景下，Tray 菜单的剪贴板操作会在主窗口的 webview 上执行 JS，而非用户交互的目标窗口。

**已有基础设施：**
- `WindowManager.controllers: Map<number, RustWebviewNodeController>` — 已存储每个窗口的 NodeController
- `WindowManager.getController(id)` — 已可获取 RustWebviewNodeController
- `RustWebviewNodeController.webviewEntries: Map<string, WebviewNodeData>` — 已管理多个 webview
- `WebviewNodeData.controller: WebviewController` — 已包含 `web_webview.WebviewController`

**关键发现：** Webview controller 已经存在于 `RustWebviewNodeController.webviewEntries` 中，无需新增存储或注册机制。只需添加一个访问方法即可。

## Goals / Non-Goals

**Goals:**
- 剪贴板/编辑操作（copy/cut/paste/selectAll/undo/redo）对目标窗口的 webview 执行 JS
- 复用 `getTargetWindow()` 的目标窗口解析逻辑（与 p1 窗口级操作一致）

**Non-Goals:**
- 不处理同一窗口内多个 webview 的焦点追踪（假设每窗口只有一个 primary webview）
- 不修改 Rust 层（tao/tauri/muda 无需变更）
- 不处理窗口类型系统（Float vs SubWindow）

## Decisions

### D1: 在 RustWebviewNodeController 添加 getPrimaryWebviewController()

在 `RustWebviewNodeController` 添加方法，返回第一个 webview 的 controller：

```typescript
getPrimaryWebviewController(): web_webview.WebviewController | null {
  const firstEntry = this.webviewEntries.values().next().value;
  return (firstEntry as WebviewNodeData)?.controller ?? null;
}
```

**理由：** Webview controller 已存在于 `webviewEntries` 中，无需新增存储。每窗口通常只有一个 primary webview，取第一个即可。

### D2: menu.ets 通过已有链路获取目标 controller

在 `execute()` 方法中，剪贴板操作通过已有链路获取目标窗口的 controller：

```typescript
// 获取目标窗口的 RustWebviewNodeController
const wm = WindowManager.getInstance();
const nodeController = wm.getController(resolvedWindowId);
// 获取其 primary webview controller，fallback 到 this.controller
const targetController = nodeController?.getPrimaryWebviewController() ?? this.controller;
```

**理由：** 复用已有的 `WindowManager.getController()` 和 `RustWebviewNodeController`，无需新增 Map 或注册机制。

### D3: 保留 this.controller 作为 fallback

不删除 `this.controller` 和 `setController()`，保留作为 fallback。

**理由：** 向后兼容：如果 `RustWebviewNodeController` 未就绪或无 webview，降级为主窗口的 controller。

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| webviewEntries 为空（webview 未创建） | Fallback 到 this.controller |
| 多 webview 场景取到非预期的 controller | 当前假设每窗口一个 primary webview，后续可扩展 |
| webviewEntries 的迭代顺序不确定 | Map 按插入顺序迭代，第一个即为主 webview |
