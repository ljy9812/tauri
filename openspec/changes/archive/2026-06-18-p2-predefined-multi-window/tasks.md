## 1. PredefinedActionExecutor showAll/bringAllToFront 实现

- [x] 1.1 在 `menu.ets` 的 `PredefinedActionExecutor` 中添加 `restoreAll()` 辅助方法：调用 `this.showAbility()` + 遍历 `WindowManager.getAllWindowIds()` 调用 `win.showWindow()`
- [x] 1.2 在 execute() switch 中将 `case 'showAll':` 从空操作改为调用 `this.restoreAll()`
- [x] 1.3 在 execute() switch 中添加 `case 'bringAllToFront':` 调用 `this.restoreAll()`

## 2. Full Test Tray 测试用例更新

- [x] 2.1 在 `tray.ts` 的 `full_test_tray` 测试中增加 `const showAll = await PredefinedMenuItem.new({ item: 'ShowAll' })` 和 `const bringAllToFront = await PredefinedMenuItem.new({ item: 'BringAllToFront' })`
- [x] 2.2 将 showAll 和 bringAllToFront 加入 Menu.new() 的 items 数组

## 3. 编译验证与测试

- [x] 3.1 HAP 构建验证：编译通过
- [x] 3.2 自动测试回归：full_test_tray 测试通过
- [x] 3.3 手动测试：Hide → tray 右键 ShowAll → 应用恢复
- [x] 3.4 手动测试：Hide → tray 右键 BringAllToFront → 应用恢复
- [x] 3.5 手动测试：有子窗口时 BringAllToFront → 子窗口恢复显示
- [x] 3.6 手动测试：应用前台时点击 ShowAll → 无副作用
