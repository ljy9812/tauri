# ohos-predefined-show-restore Specification

## Purpose
TBD - created by archiving change p2-predefined-multi-window. Update Purpose after archive.
## Requirements
### Requirement: showAll predefined action 恢复应用到前台
当用户点击 ShowAll 菜单项时，系统 SHALL 调用 `showAbility()` 恢复被隐藏的 Ability，并遍历所有窗口调用 `showWindow()` 确保窗口可见。

**平台差异说明**：macOS 的 ShowAll (`unhideAllApplications:`) 会取消隐藏所有应用，OHOS 沙箱限制只能恢复自身 Ability。

#### Scenario: hide 后点击 ShowAll 恢复
- **WHEN** 用户先点击 Hide 隐藏应用
- **WHEN** 用户点击 ShowAll 菜单项（通过 tray 右键菜单或 app menu bar）
- **THEN** 调用 `context.showAbility()` 恢复 Ability 到前台
- **THEN** 遍历 WindowManager 中所有窗口，调用 `showWindow()` 确保可见

#### Scenario: 应用未隐藏时点击 ShowAll
- **WHEN** 应用处于正常前台状态
- **WHEN** 用户点击 ShowAll 菜单项
- **THEN** showAbility() 不产生副作用（幂等操作）
- **THEN** 所有窗口保持可见

### Requirement: bringAllToFront predefined action 恢复应用并显示所有窗口
当用户点击 BringAllToFront 菜单项时，系统 SHALL 调用 `showAbility()` 恢复 Ability，并遍历所有窗口调用 `showWindow()` 显示。

**平台差异说明**：macOS 的 BringAllToFront (`arrangeInFront:`) 会将当前应用所有窗口重新排列到其他应用之上。OHOS 无等价 API（`setWindowTopmost` 需系统权限），因此语义降级为与 ShowAll 等价。

#### Scenario: hide 后点击 BringAllToFront 恢复
- **WHEN** 用户先点击 Hide 隐藏应用
- **WHEN** 用户点击 BringAllToFront 菜单项
- **THEN** 调用 `context.showAbility()` 恢复 Ability 到前台
- **THEN** 遍历所有窗口调用 `showWindow()` 确保可见

#### Scenario: 有最小化子窗口时点击 BringAllToFront
- **WHEN** 主窗口可见，子窗口被最小化
- **WHEN** 用户点击 BringAllToFront 菜单项
- **THEN** 子窗口通过 `showWindow()` 恢复显示

### Requirement: Menu 和 Tray 两条路径共享 predefined 逻辑
PredefinedActionExecutor.execute() 同时被 Menu 和 Tray 路径调用，showAll/bringAllToFront 的实现 MUST 在两条路径上都正确工作。

#### Scenario: 通过 app menu bar 点击 ShowAll
- **WHEN** 应用有 menu bar 且包含 ShowAll 菜单项
- **WHEN** 用户通过 menu bar 点击 ShowAll
- **THEN** MenuManager.handleItemClick() → executor.execute('showAll') 正确恢复

#### Scenario: 通过 tray 右键菜单点击 ShowAll
- **WHEN** tray 右键菜单包含 ShowAll 菜单项
- **WHEN** 用户通过 tray 菜单点击 ShowAll
- **THEN** executor.execute('showAll') 正确恢复（无 targetWindowId 参数）

### Requirement: Full Test Tray 包含 ShowAll 和 BringAllToFront 测试项
Full Test Tray 自动测试 MUST 在菜单中包含 ShowAll 和 BringAllToFront predefined 项。

#### Scenario: Full Test Tray 创建包含 ShowAll 和 BringAllToFront
- **WHEN** 运行 full_test_tray 自动测试
- **THEN** 菜单中包含 `PredefinedMenuItem.new({ item: 'ShowAll' })`
- **THEN** 菜单中包含 `PredefinedMenuItem.new({ item: 'BringAllToFront' })`
- **THEN** Tray 创建和销毁成功

