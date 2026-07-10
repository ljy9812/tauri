## Why

OHOS predefined menu 中 6 个剪贴板/编辑操作（copy/cut/paste/selectAll/undo/redo）使用硬编码的主窗口 webview controller（`this.controller`），在多窗口场景下无法对正确的目标窗口执行 JS 操作。p1/p2 已修复了窗口级操作（hide/close/minimize/maximize/fullscreen/recover/showAll/bringAllToFront）的目标窗口解析，但剪贴板操作仍遗漏。作为 predefined 多窗支持的最后一步，需要修复剪贴板操作的目标窗口解析。

## What Changes

- **RustWebviewNodeController** 增加 `getPrimaryWebviewController()` 访问方法
- **copy/cut/paste/selectAll/undo/redo** 改用目标窗口的 webview controller 执行 JS（Window 级操作）
- 目标窗口解析复用 `getTargetWindow()`（与 p1/p2 窗口级操作一致）

## Capabilities

### New Capabilities
- `ohos-predefined-clipboard`: 剪贴板/编辑操作使用目标窗口的 webview controller

### Modified Capabilities
- `ohos-predefined-window-ops`: 补充 clipboard 操作的目标窗口解析规约

## Impact

- **ArkTS 层**：`DefaultWebview.ets`（新增访问方法）、`menu.ets`（6 个 case 改用目标 controller）
- **Rust 层**：无修改
