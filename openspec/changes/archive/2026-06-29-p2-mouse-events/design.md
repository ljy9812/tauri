## Context

p1-mouse-events 已完成基础鼠标事件链路。触控板在 OHOS 上被系统映射为鼠标事件 + 轴事件，但轴事件中携带的捏合缩放值和输入源信息未被提取。

OHOS NDK 提供的 ArkUI 级 API：
- `OH_ArkUI_AxisEvent_GetPinchAxisScaleValue` — 提取双指捏合缩放比例
- `OH_ArkUI_UIInputEvent_GetSourceType` — 区分 Mouse(1)/TouchScreen(2)/Touchpad(3)

## Goals / Non-Goals

**Goals:**
- 从轴事件中提取捏合 scale 并传递给 WebView（Ctrl+Wheel = 缩放）
- 区分触控板/鼠标输入源，使用对应的 ScrollDelta 类型
- 手动测试区域可视化区分 scroll 和 pinch-zoom

**Non-Goals:**
- 旋转手势（RotationGesture）— 当前 NDK 未提供对应轴值 API
- 三指/四指手势 — OHOS 系统不上报多指信息
- 键盘修饰键状态（Ctrl/Alt/Shift）— 需要 `getModifierKeyState` ArkTS API，不在 NDK 层

## Decisions

### D1: 捏合缩放传递方式

**决策**: 将 pinch_scale 转换为 `MouseWheel` + `ModifiersState::CONTROL`

**理由**: 浏览器标准将 Ctrl+Wheel 解释为页面缩放（CSS zoom），无需新增 tao 事件类型。Web 应用无需任何改动即可响应捏合缩放。

**替代方案**:
- 新增 `WindowEvent::PinchGesture { scale }` — 需要修改 tao 事件枚举，影响所有平台
- 通过 ArkTS `onPinch` 回调传递 — 需要修改 ArkTS 层，链路更长

### D2: 触控板滚动 delta 类型

**决策**: 触控板 → `PixelDelta`（像素），鼠标滚轮 → `LineDelta`（行）

**理由**: OHOS 触控板轴事件上报位移像素，鼠标滚轮上报角度。与 macOS 行为一致（触控板 → PixelDelta，滚轮 → LineDelta）。

### D3: 输入源获取位置

**决策**: 仅在 `dispatch_axis_event` 中获取 source_type

**理由**: `OH_ArkUI_UIInputEvent_GetSourceType` 需要 `ArkUI_UIInputEvent` 指针，仅在 `RegisterUIInputEventCallback` 回调中可用。鼠标事件的 `DispatchMouseEvent` 回调不提供此指针。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| Ctrl+Wheel 缩放粒度固定（±1.0） | 可后续根据 pinch_scale 差值动态调整 |
| 鼠标事件无 source_type | 触控板单指操作映射为鼠标事件，WebView 层已自动处理 |
