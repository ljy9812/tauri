## 1. 光标位置跟踪

- [x] 1.1 添加静态原子变量 `LAST_CURSOR_POSITION_X: AtomicU64` 和 `LAST_CURSOR_POSITION_Y: AtomicU64`
- [x] 1.2 在 `handle_mouse_event` 的 `MouseAction::Move` 分支中更新原子变量（`f64::to_bits()`）
- [x] 1.3 `EventLoopWindowTarget::cursor_position()` 读取原子变量并返回 `PhysicalPosition`（`f64::from_bits()`）
- [x] 1.4 `Window::cursor_position()` 读取原子变量并返回 `PhysicalPosition`

## 2. 构建验证

- [x] 2.1 `cargo check --target aarch64-unknown-linux-ohos` 通过
- [x] 2.2 设备端构建部署，cursorPosition 测试通过（211 pass / 3 fail，无回归）

## 3. 测试用例

- [x] 3.1 自动测试：`@tauri-apps/api/window.cursorPosition` — 验证 API 返回有效坐标
- [x] 3.2 手动测试：`Get Cursor Position` 按钮 — 点击后显示当前光标坐标
