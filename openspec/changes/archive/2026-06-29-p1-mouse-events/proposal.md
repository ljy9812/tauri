## Why

OHOS 2in1 桌面设备（PC/平板二合一）已广泛使用鼠标外设，但当前 Tauri OHOS 平台实现中鼠标事件完全未实现（`tao/src/platform_impl/ohos/mod.rs:122` 标注 `TODO mouse events`）。这导致所有依赖鼠标交互的功能（右键菜单、悬停提示、拖拽操作、滚轮滚动）在 OHOS 桌面模式下无法工作。OHOS NDK 已提供完整的鼠标事件 FFI 支持（`OH_NativeXComponent_MouseEvent` + `RegisterMouseEventCallback`），binding 层也有半成品枚举定义，只需补齐整条事件链路。

## What Changes

- **ohos-xcomponent-binding crate**：新增 `MouseEventData` 结构体、`dispatch_mouse_event` 回调分发、`on_mouse_event()` 注册方法，以及 `DispatchHoverEvent` 回调支持
- **openharmony-ability**：`InputEvent` 枚举新增 `MouseEvent` 变体，xcomponent 渲染层注册鼠标回调将 NDK 事件桥接到 Rust 事件循环
- **tao OHOS platform_impl**：在 `handle_input_event` 中处理 `InputEvent::MouseEvent`，转换为 `WindowEvent::CursorMoved` / `MouseInput` / `CursorEntered` / `CursorLeft`；处理 `DispatchHoverEvent` 实现光标进入/离开检测
- **不支持的功能**：`MouseWheel`（滚轮）在当前 NDK FFI（ohos-xcomponent-sys v0.0.2）中无 scroll delta 字段，需 stub 处理，后续可通过 API 20+ 的 `OH_NativeXComponent_GetExtraMouseEventInfo` 或 ArkTS `onMouse` 事件补齐

## Capabilities

### New Capabilities
- `ohos-mouse-input`: 鼠标移动/点击事件的 NDK 回调注册、数据解析和事件分发（MouseEvent::Press/Release/Move → CursorMoved/MouseInput）
- `ohos-hover-events`: 鼠标悬停事件（DispatchHoverEvent isHover → CursorEntered/CursorLeft）

### Modified Capabilities
<!-- 无现有 capability 需要修改 -->

## Impact

- **依赖**：`ohos-xcomponent-binding` 需要发布新版本（或 fork 到 openharmony-ability 仓库），`ohos-xcomponent-sys` v0.0.2 已包含所需的 FFI 定义
- **API 兼容性**：鼠标事件回调自 OHOS API 9 起可用，满足当前最低 API 12 要求
- **受影响代码层**：ohos-xcomponent-binding（6 文件）→ openharmony-ability（2 文件）→ tao（1 文件），共 ~9 文件
- **平台限制**：仅在 2in1 桌面设备 + `cfg(desktop)` 模式下鼠标事件有意义；手机端鼠标事件回调不会被触发
- **线程安全**：鼠标回调在 XComponent 线程触发，通过现有事件循环机制（`borrow_mut` + handler call）传递到主线程，与触摸事件路径一致
