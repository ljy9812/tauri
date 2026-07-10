## ADDED Requirements

### Requirement: 剪贴板操作使用目标窗口的 webview controller

copy/cut/paste/selectAll/undo/redo 操作 SHALL 使用 `getTargetWindow()` 解析目标窗口后，通过已有的 `WindowManager.getController(windowId)` → `RustWebviewNodeController.getPrimaryWebviewController()` 链路获取该窗口的 webview controller 执行 JS。

#### Scenario: Tray 菜单 Copy 操作在子窗口上执行
- **WHEN** 用户最后交互的窗口是子窗口（id > 0）
- **WHEN** 用户点击 Tray 菜单的 Copy 菜单项
- **THEN** 通过 `getTargetWindow()` 获取子窗口 ID
- **THEN** 通过 `WindowManager.getController(id)` 获取子窗口的 `RustWebviewNodeController`
- **THEN** 调用 `getPrimaryWebviewController()` 获取子窗口的 `web_webview.WebviewController`
- **THEN** 在子窗口的 webview 上执行 `window.getSelection().toString()` 获取选中文本
- **THEN** 将选中文本写入系统剪贴板

#### Scenario: Window Menu Bar Paste 操作在当前窗口上执行
- **WHEN** 用户点击 Window Menu Bar 的 Paste 菜单项（targetWindowId 已提供）
- **THEN** 通过 `getTargetWindow(targetWindowId)` 获取目标窗口
- **THEN** 获取该窗口的 webview controller
- **THEN** 在该窗口的 webview 上执行 `document.execCommand("insertText", ...)` 插入剪贴板内容

#### Scenario: 目标窗口 controller 未就绪时 fallback
- **WHEN** 目标窗口的 `RustWebviewNodeController` 不存在或无 webview
- **THEN** 降级使用 `this.controller`（主窗口的 controller）执行 JS
