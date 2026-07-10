## 1. RustWebviewNodeController 添加访问方法

- [x] 1.1 在 `RustWebviewNodeController` 中添加 `getPrimaryWebviewController(): web_webview.WebviewController | null` 方法

## 2. menu.ets 剪贴板操作改用目标窗口 controller

- [x] 2.1 在 `PredefinedActionExecutor` 中添加 `getTargetController(windowId: number)` 方法，通过 `WindowManager.getController(id)` → `getPrimaryWebviewController()` 获取，fallback 到 `this.controller`
- [x] 2.2 修改 `copy` case：使用 `getTargetController(resolvedWindowId)` 替代 `this.controller`
- [x] 2.3 修改 `cut` case：同上
- [x] 2.4 修改 `paste` case：同上
- [x] 2.5 修改 `selectAll` case：同上
- [x] 2.6 修改 `undo` case：同上
- [x] 2.7 修改 `redo` case：同上

## 3. 验证

- [x] 3.1 编译部署（OHOS_DEVICE_TYPE=desktop）
- [x] 3.2 手动测试：点击子窗口 → Tray 菜单 Copy，确认复制的是子窗口的选中文本
- [ ] 3.3 手动测试：点击主窗口 → Tray 菜单 Paste，确认粘贴到主窗口（跳过：api demo 无 READ_PASTEBOARD 权限）
