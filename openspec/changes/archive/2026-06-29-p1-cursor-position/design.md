## Context

tao OHOS platform_impl 中两处 `cursor_position()` 硬编码返回 `(0, 0)`:
- `EventLoopWindowTarget::cursor_position()` (line 695)
- `Window::cursor_position()` (line 1050)

OHOS NDK 不提供 `GetCursorPos` 类似的全局光标查询 API。但 `DispatchMouseEvent` 回调的 `OH_NativeXComponent_MouseEvent` 结构体包含 `x, y`（窗口坐标）和 `screenX, screenY`（屏幕坐标），已在 `handle_mouse_event` 的 `MouseAction::Move` 分支中使用。

## Goals / Non-Goals

**Goals:**
- `cursor_position()` 返回最近一次鼠标移动的位置
- 线程安全（使用原子变量）

**Non-Goals:**
- 全局光标位置查询（OHOS 无此 API）
- `set_cursor_position()` — OHOS 无此 API，保持 `NotSupported`
- 触摸位置跟踪 — 触摸事件已作为 `Touch` 事件单独处理

## Decisions

### D1: 存储方式

**决策**: 使用 `AtomicU64` 存储 f64 的 bit representation

**理由**:
- 原子操作无锁，读写不会阻塞
- f64 → u64 → f64 无损转换（`to_bits()` / `from_bits()`）
- 与现有 `HAS_FOCUS: AtomicBool` 模式一致

**替代方案**:
- `Mutex<(f64, f64)>` — 有锁开销，且在事件处理中可能导致死锁
- `Cell<(f64, f64)>` — 非 Sync，无法跨线程访问

### D2: 坐标系统

**决策**: 存储窗口相对坐标（`x, y`），与 `CursorMoved` 事件一致

**理由**: `cursor_position()` 在其他平台上返回窗口相对坐标（Windows `GetCursorPos` 返回屏幕坐标后会减去窗口偏移）。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 光标未移动前返回 (0, 0) | 这是 OHOS 平台限制，与 Android 行为一致 |
| 仅跟踪 XComponent 内的光标 | OHOS 应用只有 XComponent，无其他原生区域 |
