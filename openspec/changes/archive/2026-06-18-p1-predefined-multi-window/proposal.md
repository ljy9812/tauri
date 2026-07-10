## Why

OHOS predefined menu 的窗口级操作（hide/closeWindow/minimize/maximize/fullscreen/recover）当前存在两个问题：
1. hide/closeWindow 映射到 `win.minimize()`，与 Windows/macOS 语义不对齐
2. 所有操作硬编码到主窗口，多窗口场景下无法对正确的目标窗口生效

需要修复语义并对齐目标窗口解析机制。

## What Changes

- **hide** → `context.hideAbility(want)` 隐藏整个应用到后台（App 级操作）
- **close (子窗口 id>0)** → `destroyWindow()` 正常关闭窗口
- **close (主窗口 id=0)** → `context.hideAbility(want)` 隐藏到后台
- **托盘图标点击** → `context.showAbility(want)` 自动恢复
- **minimize/maximize/fullscreen/recover** → 增加目标窗口解析（`getTargetWindow()`）
- **quit** → 保持 `terminateSelf()` 不变（App 级操作）

## Capabilities

### New Capabilities
- `ohos-predefined-window-ops`: OHOS predefined menu 窗口级操作的目标窗口解析与语义修正

## Impact

- **ArkTS 层**：`menu.ets`、`StatusBarUtils.ets`、`NativeAbility.ets`
- **Rust 层**：无修改
