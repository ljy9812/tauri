## Why

p1-mouse-events 实现了基础鼠标事件（移动/点击/悬停/滚轮），但触控板特有功能尚未覆盖：
- 触控板双指捏合（pinch-to-zoom）的 scale 值未从 NDK 提取
- 无法区分输入源（触控板 vs 物理鼠标），导致滚动事件一律使用 LineDelta（行），触控板应使用 PixelDelta（像素）
- 手动测试区域未区分 scroll 和 pinch-zoom 事件

## What Changes

- **openharmony-ability**: AxisEventData 新增 `pinch_scale` 和 `source_type` 字段；dispatch_axis_event 调用 `OH_ArkUI_AxisEvent_GetPinchAxisScaleValue` 和 `OH_ArkUI_UIInputEvent_GetSourceType`；新增 `InputSourceType` 枚举
- **tao**: handle_axis_event 根据 source_type 选择 PixelDelta（触控板）或 LineDelta（鼠标）；pinch_scale ≠ 0/1.0 时发出 Ctrl+MouseWheel（WebView 解释为缩放）
- **tauri 前端测试**: 新增 Ctrl+Wheel 自动测试；手动跟踪区域区分 scroll/pinch-zoom 并以紫色高亮显示

## Capabilities

### New Capabilities
- `ohos-touchpad-enhancements`: 触控板捏合缩放提取、输入源区分、手动测试增强

### Modified Capabilities

## Impact

- **依赖**: 无新增（ohos-xcomponent-sys 和 ohos-arkui-sys 已在 p1 中引入）
- **API 兼容性**: 使用 API 12 已有的 NDK 函数
- **受影响代码层**: openharmony-ability（1 文件）→ tao（1 文件）→ tauri（2 文件）
- **行为变化**: 触控板滚动从 LineDelta 变为 PixelDelta，更精确；捏合操作触发 WebView 缩放
