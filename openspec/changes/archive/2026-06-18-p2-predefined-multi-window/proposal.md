## Why

OHOS predefined menu 中 `showAll` 和 `bringAllToFront` 当前为空操作（no-op），用户 hide 应用后无法通过菜单恢复。作为 predefined 多窗支持的一部分，需要实现这两个 App 级恢复操作。

## What Changes

- **showAll** → `showAbility()` + 遍历所有窗口调用 `showWindow()` 恢复显示（App 级操作）
- **bringAllToFront** → `showAbility()` + 遍历所有窗口调用 `showWindow()` 恢复显示（App 级操作，OHOS 语义与 showAll 等价）
- **Full Test Tray 测试** → 增加 ShowAll 和 BringAllToFront 菜单项

## 与 macOS 行为差异说明

### macOS
- **ShowAll** (`unhideAllApplications:`): 取消隐藏**所有应用**（不仅当前 app），系统级操作
- **BringAllToFront** (`arrangeInFront:`): 将**当前应用**的所有窗口重新排列到其他应用窗口之上

### OHOS 限制
- OHOS 沙箱模型只允许控制自身 Ability，无法"取消隐藏所有应用"
- OHOS 无 `arrangeInFront:` 等价 API（`setWindowTopmost()` 需要 WINDOW_TOPMOST 系统权限）
- 因此 ShowAll 和 BringAllToFront 在 OHOS 上语义等价：恢复自身 Ability + 显示所有窗口

### 差异合理性
- Windows 也不支持这两个操作（muda 中标记为 Unsupported）
- OHOS 作为类移动端平台，与 Windows 类似缺少全局窗口管理能力，降级为"恢复自身"是合理的
- 对用户而言，核心需求是"hide 后能恢复"，ShowAll/BringAllToFront 都能满足此需求

## Capabilities

### New Capabilities
- `ohos-predefined-show-restore`: OHOS predefined menu ShowAll/BringAllToFront 恢复行为

### Modified Capabilities
- `ohos-predefined-window-ops`: 补充 showAll/bringAllToFront 行为规约

## Impact

- **ArkTS 层**：`menu.ets`（execute switch）
- **前端测试**：`tray.ts`（Full Test Tray 增加菜单项）
- **Rust 层**：无修改（muda/tauri 已有完整支持）
