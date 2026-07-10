## 1. Ability 层 — 捏合 scale + 输入源

- [x] 1.1 新增 `InputSourceType` 枚举（Mouse/TouchScreen/Touchpad/Joystick/Keyboard），实现 `From<i32>`
- [x] 1.2 `AxisEventData` 新增 `pinch_scale: f32` 和 `source_type: InputSourceType` 字段
- [x] 1.3 `dispatch_axis_event` 调用 `OH_ArkUI_AxisEvent_GetPinchAxisScaleValue` 提取捏合值
- [x] 1.4 `dispatch_axis_event` 调用 `OH_ArkUI_UIInputEvent_GetSourceType` 获取输入源
- [x] 1.5 跳过无滚动 delta 且无捏合数据的事件

## 2. tao 平台实现 — PixelDelta + 捏合缩放

- [x] 2.1 `handle_axis_event` 根据 source_type 选择 PixelDelta（触控板）或 LineDelta（鼠标）
- [x] 2.2 pinch_scale > 1.0 时发出 `MouseWheel { delta: LineDelta(0, 1), modifiers: CONTROL }`（zoom in）
- [x] 2.3 pinch_scale < 1.0 且 != 0.0 时发出 `MouseWheel { delta: LineDelta(0, -1), modifiers: CONTROL }`（zoom out）

## 3. 前端测试

- [x] 3.1 新增 `DOM WheelEvent.ctrlKey (pinch zoom simulation)` 自动测试
- [x] 3.2 手动跟踪区域区分 scroll/pinch-zoom 事件类型
- [x] 3.3 pinch-zoom 事件以紫色加粗显示

## 4. 构建验证

- [x] 4.1 `cargo check --target aarch64-unknown-linux-ohos` 通过
- [x] 4.2 设备端测试 210 pass / 2 fail（无回归）
- [x] 4.3 手动验证触控板双指滑动显示 scroll、捏合显示 pinch-zoom
