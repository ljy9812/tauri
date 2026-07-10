# Mouse Events & Touchpad 适配计划

**创建时间**：2026-06-27
**完成时间**：2026-06-29
**功能描述**：为 OHOS 平台实现鼠标事件 + 触控板增强支持，使 2in1 桌面设备上的鼠标/触控板操作能正确传递到 Tauri 应用。

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 归档位置 |
|-------|------|----------------|------|---------|
| 1 | 鼠标事件 + 悬停 + 滚轮 | p1-mouse-events | ✓ 已归档 | `archive/2026-06-29-p1-mouse-events/` |
| 2 | 触控板增强（捏合 + 输入源） | p2-mouse-events | ✓ 已归档 | `archive/2026-06-29-p2-mouse-events/` |

## Phase 1: 鼠标事件 + 悬停 + 滚轮

**归档**: `archive/2026-06-29-p1-mouse-events/`

### 实现内容
- `MouseEventData` 结构体 + `MouseAction` 枚举（Press/Release/Move/HoverEnter/HoverLeave）
- NDK 回调：`DispatchMouseEvent`、`DispatchHoverEvent`、`RegisterUIInputEventCallback(AXIS)`
- `InputEvent::MouseEvent` 和 `InputEvent::AxisEvent` 变体
- tao: CursorMoved / MouseInput / CursorEntered / CursorLeft / MouseWheel
- 前端测试：MouseEvent.dispatch、MouseEvent.coordinates、WheelEvent.dispatch
- 手动测试：鼠标跟踪区域

### 修改文件
- `openharmony-ability/crates/ability/src/input/mouse_event.rs` (新建)
- `openharmony-ability/crates/ability/src/input/mod.rs`
- `openharmony-ability/crates/ability/src/render/xcomponent.rs`
- `openharmony-ability/Cargo.toml` + `crates/ability/Cargo.toml`
- `tao/src/platform_impl/ohos/mod.rs`
- `tauri/examples/api/src/lib/tests/core.ts`
- `tauri/examples/api/src/views/TestRunner.svelte`
- `tauri/examples/api/src-tauri/capabilities/run-app.json`

### 测试结果
209 pass / 2 fail（新增 3 个鼠标/滚轮测试）

## Phase 2: 触控板增强（捏合缩放 + 输入源区分）

**归档**: `archive/2026-06-29-p2-mouse-events/`

### 实现内容
- `InputSourceType` 枚举（Mouse/TouchScreen/Touchpad/Joystick/Keyboard）
- `AxisEventData` 新增 `pinch_scale` + `source_type` 字段
- `OH_ArkUI_AxisEvent_GetPinchAxisScaleValue` 提取捏合值
- `OH_ArkUI_UIInputEvent_GetSourceType` 获取输入源
- tao: 触控板 → PixelDelta，鼠标 → LineDelta
- tao: 捏合 → Ctrl+MouseWheel（WebView 缩放）
- 前端测试：WheelEvent.ctrlKey（捏合模拟）
- 手动测试：scroll/pinch-zoom 区分显示

### 修改文件
- `openharmony-ability/crates/ability/src/input/mouse_event.rs`
- `tao/src/platform_impl/ohos/mod.rs`
- `tauri/examples/api/src/lib/tests/core.ts`
- `tauri/examples/api/src/views/TestRunner.svelte`

### 测试结果
210 pass / 2 fail（新增 1 个 Ctrl+Wheel 测试）

## PR 链接

| 仓库 | PR |
|------|-----|
| openharmony-ability | https://github.com/Eulogizethesun/openharmony-ability/pull/34 |
| tao | https://github.com/Eulogizethesun/tao/pull/10 |
| tauri | https://github.com/Eulogizethesun/tauri/pull/53 |
