## Why

OHOS 平台的 `cursor_position()` API 硬编码返回 `(0, 0)`，导致所有依赖光标位置的功能无法工作（如 tooltip 定位、右键菜单弹出位置、拖拽起始点）。OHOS NDK 不提供全局光标查询 API，但鼠标移动事件中已携带光标坐标，可通过跟踪最近位置来实现。

## What Changes

- **tao OHOS platform_impl**: 添加静态原子变量存储最近光标位置；`handle_mouse_event` 的 Move 分支更新位置；`cursor_position()` 和 `EventLoopWindowTarget::cursor_position()` 读取存储值

## Capabilities

### New Capabilities
- `ohos-cursor-position`: 从鼠标移动事件跟踪光标位置，使 `cursor_position()` 返回实际坐标

### Modified Capabilities

## Impact

- **受影响代码**: `tao/src/platform_impl/ohos/mod.rs`（1 个文件，~20 行）
- **局限性**: 光标未移动前仍返回 (0, 0)；仅跟踪鼠标在 XComponent 上方时的位置
- **无新依赖**: 使用已有的 `std::sync::atomic`
