# Predefined Hide/Close 适配计划

**创建时间**：2026-06-15
**功能描述**：OHOS predefined menu hide/close/showAll/bringAllToFront 语义对齐

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | hide/close/minimize 语义修正 | p1-predefined-multi-window | ✓ 已归档 | ArkTS | 3 | 设备端手动测试 |
| 2 | ShowAll/BringAllToFront | p2-predefined-multi-window | ✓ 已归档 | ArkTS + 前端测试 | 2 | 设备端手动测试 + auto test |

## Phase 详细说明

### Phase 1: hide/close/minimize 语义修正 (p1)
- **目标**：hide → hideAbility(), close 主窗口 → hideAbility(), close 子窗口 → destroyWindow()
- **文件列表**：menu.ets, StatusBarUtils.ets, WindowManager.ets
- **状态**：✓ 已完成（实现 + 验证通过）

### Phase 2: ShowAll/BringAllToFront 实现 (p2)
- **目标**：实现 showAll/bringAllToFront predefined action，增加测试
- **文件列表**：menu.ets, tray.ts
- **依赖**：Phase 1 完成
- **状态**：✓ 已完成（实现 + 验证通过）
