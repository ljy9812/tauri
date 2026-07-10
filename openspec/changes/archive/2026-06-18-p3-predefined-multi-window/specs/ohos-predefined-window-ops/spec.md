## MODIFIED Requirements

### Requirement: 各操作的层级定义

OHOS 只有一个 tray icon（应用级入口），因此 predefined 菜单操作 MUST 明确区分 app 级与 window 级语义。对标 macOS 响应者链的设计：

| 操作 | 层级 | macOS 对标 | 说明 |
|------|------|-----------|------|
| Hide | **App 级** | `NSApplication.hide:` | 隐藏整个应用（所有窗口） |
| Minimize | **Window 级** | `NSWindow.performMiniaturize:` | 最小化当前焦点窗口 |
| CloseWindow | **Window 级** | `NSWindow.performClose:` | 关闭当前焦点窗口 |
| Maximize | **Window 级** | `NSWindow.performZoom:` | 最大化当前焦点窗口 |
| Fullscreen | **Window 级** | `NSWindow.toggleFullScreen:` | 全屏当前焦点窗口 |
| Recover | **Window 级** | 再次 `performZoom:` | 从最大化/全屏恢复 |
| Copy | **Window 级** | `copy:` selector | 复制目标窗口 webview 的选中文本 |
| Cut | **Window 级** | `cut:` selector | 剪切目标窗口 webview 的选中文本 |
| Paste | **Window 级** | `paste:` selector | 粘贴剪贴板内容到目标窗口 webview |
| SelectAll | **Window 级** | `selectAll:` selector | 全选目标窗口 webview 的内容 |
| Undo | **Window 级** | `undo:` selector | 撤销目标窗口 webview 的操作 |
| Redo | **Window 级** | `redo:` selector | 重做目标窗口 webview 的操作 |

- App 级操作不依赖窗口焦点，直接对整个 Ability 生效
- Window 级操作 MUST 确定目标窗口，通过 `lastUserInteractedWindow` 追踪机制获取（基于 onTouch 事件）
- 剪贴板/编辑操作属于 Window 级，SHALL 在目标窗口的 webview controller 上执行 JS

#### Scenario: 剪贴板操作在子窗口上执行
- **WHEN** 用户最后交互的窗口是子窗口（id > 0）
- **WHEN** 用户点击 Tray 菜单的 Copy/Cut/SelectAll/Undo/Redo 菜单项
- **THEN** 操作 SHALL 在子窗口的 webview controller 上执行 JS
