## Context

p1 已实现 hide/close/minimize 的正确语义。`showAll` 和 `bringAllToFront` 在 menu.ets 中为空操作，需要实现。

**当前状态**：
- `menu.ets` switch 中 `case 'showAll': break;` — 空操作（与 hideOthers 合并）
- `menu.ets` switch 中无 `case 'bringAllToFront'` — 完全缺失（落入 default 分支）

**tray icon 左键恢复已正常工作**：无 QuickOperation 模式（abilityName=""）时，`StatusBarUtils.iconClickHandler` 调用 `startAbility()` 可以正常恢复被 `hideAbility()` 隐藏的应用。

**两条 predefined 执行路径**（共享 PredefinedActionExecutor.execute()）：
```
Menu path:  MenuPopup → MenuManager.handleItemClick() → executor.execute(type, metadata, windowId)
Tray path:  ArkHelper.executePredefinedAction() → executor.execute(actionType)  // 无 windowId
```

## Goals / Non-Goals

**Goals:**
- 实现 showAll 和 bringAllToFront 恢复应用到前台
- 在 Full Test Tray 自动测试中覆盖这两个 predefined 类型
- 明确 OHOS 与 macOS 的行为差异

**Non-Goals:**
- 不实现"取消隐藏所有应用"（OHOS 沙箱限制）
- 不实现窗口置顶到其他应用之上（需 WINDOW_TOPMOST 系统权限）
- 不修改 Rust/muda 层代码（已完整支持）
- 不修改 StatusBarUtils.ets（tray 左键恢复已正常工作）

## Decisions

### D1: showAll 和 bringAllToFront 在 OHOS 上语义等价

**选择**：两者都执行 `showAbility()` + 遍历窗口 `showWindow()`

**理由**：
- macOS ShowAll (`unhideAllApplications:`) 是系统级操作，OHOS 无此能力
- macOS BringAllToFront (`arrangeInFront:`) 需要跨应用窗口管理能力，OHOS 的 `setWindowTopmost()` 需系统权限
- Windows 也不支持这两个操作（muda 标记为 Unsupported）
- 对用户核心需求"hide 后恢复"，两者语义等价是合理的降级

**替代方案**：将两者标记为 Unsupported → 但用户需要一种从菜单恢复应用的方式

### D2: showAll/bringAllToFront 实现为共享辅助方法

**选择**：在 `PredefinedActionExecutor` 中提取 `restoreAll()` 方法，showAll 和 bringAllToFront 都调用它

**理由**：
- 两者语义等价，避免代码重复
- 未来如果需要区分行为，只需修改调用点

```typescript
private async restoreAll(): Promise<void> {
  // 1. showAbility 恢复 Ability
  if (this.context) {
    try { await this.showAbility(); } catch (e) { /* warn */ }
  }
  // 2. 遍历所有窗口 showWindow
  try {
    const wm = WindowManager.getInstance();
    for (const id of wm.getAllWindowIds()) {
      const win = wm.getWindow(id);
      if (win) {
        try { await win.showWindow(); } catch (e) { /* warn */ }
      }
    }
  } catch (e) { /* warn */ }
}
```

### D3: showAbility() 幂等安全

**选择**：不增加"是否已隐藏"条件判断，直接调用

**理由**：
- `showAbility()` 在 Ability 未隐藏时调用不产生副作用
- 简化代码逻辑

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| showAbility() 失败 | try/catch + hilog warning |
| showWindow() 对已可见窗口调用可能报错 | try/catch 包裹，忽略错误 |
| bringAllToFront 无法将窗口置其他应用之上 | 文档标注平台差异，用户可接受 |
| showAll 无法取消隐藏其他应用 | 同上 |

## 平台差异总结

| 操作 | macOS | Windows | OHOS |
|------|-------|---------|------|
| ShowAll | 取消隐藏所有应用 | Unsupported | 恢复自身 Ability + 显示所有窗口 |
| BringAllToFront | 当前应用窗口置顶 | Unsupported | 恢复自身 Ability + 显示所有窗口（同 ShowAll） |
