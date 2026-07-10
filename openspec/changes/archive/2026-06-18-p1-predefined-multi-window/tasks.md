## 1. PredefinedActionExecutor hide/close 重写

- [x] 1.1 在 `menu.ets` 的 `PredefinedActionExecutor` 中添加 `hideAbility()` 和 `showAbility()` 辅助方法，构造 `Want { bundleName, abilityName }` 并调用 `this.context.hideAbility(want)` / `this.context.showAbility(want)`
- [x] 1.2 拆分 `case 'minimize'` / `case 'hide'` / `case 'close'` 的 fallthrough，改为独立分支
- [x] 1.3 `case 'hide'`：调用 `hideAbility()` 替代 `minimizeWithRestoreGuard()`
- [x] 1.4 `case 'close'`：根据 `targetWindowId` 判断 — 子窗口 (id > 0) 走 `notifyWindowClose + removeWindow + destroyWindow` 路径，主窗口 (id = 0) 走 `hideAbility()` 路径
- [x] 1.5 `case 'minimize'`：保持 `minimizeWithRestoreGuard(win)` 不变

## 2. 托盘图标点击自动恢复

- [x] 2.1 在 `StatusBarUtils.ets` 的 `iconClickHandler` 中，在转发给 Rust TSFN 之前/之后，调用 `getAbilityContext().showAbility(want)` 恢复应用
- [x] 2.2 构造 `Want` 使用已有的 `abilityContext.abilityInfo.bundleName` + `currentAbilityName`
- [x] 2.3 添加 try/catch 容错：showAbility 失败时记录 hilog warning，不影响 Rust 事件转发

## 3. 编译验证与测试

- [x] 3.1 HAP 构建验证：`hvigorw assembleHap` 编译通过
- [x] 3.2 设备部署：安装 HAP 到设备
- [x] 3.3 手动测试：Menu → Hide → 应用隐藏 → 点击托盘 → 应用恢复
- [x] 3.4 手动测试：Menu → Close（主窗口）→ 应用隐藏 → 点击托盘 → 应用恢复
- [x] 3.5 手动测试：Menu → Minimize → 最小化到最近任务（行为不变）
- [x] 3.6 手动测试：Menu → Quit → 应用退出（行为不变）
- [x] 3.7 手动测试：应用前台时点击托盘图标 → 无副作用
- [x] 3.8 autotest 回归：现有测试不受影响

> 注：3.x 任务需要在 OHOS 设备上手动验证

## 4. onTouch 迁移到页面根容器（D8）

- [x] 4.1 `MainPage.ets`：根 `Stack` 加 `.onTouch()` → `WindowManager.setUserInteractedWindow(0)`
- [x] 4.2 `FloatPage.ets`：根 `Stack` 加 `.onTouch()` → `WindowManager.setUserInteractedWindow(this.windowId)`
- [x] 4.3 `DefaultXComponent.ets`：移除 `.onTouch()` 及 `HitTestMode.Transparent`（不再需要）
- [x] 4.4 `WindowManager.ets`：更新 `setUserInteractedWindow` 注释（`DefaultXComponent.onTouch` → `page root onTouch`）
- [x] 4.5 手动测试：点击主窗口 MenuBar → Tray 菜单 Copy → 确认操作在主窗口
- [x] 4.6 手动测试：点击子窗口 MenuBar → Tray 菜单 Copy → 确认操作在子窗口
