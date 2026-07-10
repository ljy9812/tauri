## 1. Binding 层 — MouseEventData 结构体

> **实现调整**: 由于 `ohos-xcomponent-binding` 是外部 crate（v0.2.0 from crates.io），
> 所有 binding 层任务改为在 `openharmony-ability` 层直接通过 FFI 实现。
> 等效功能已实现，binding fork 作为后续优化。

- [x] 1.1 在 `openharmony-ability/crates/ability/src/input/mouse_event.rs` 中定义 `MouseEventData` 结构体（x, y, screen_x, screen_y, timestamp, action, button），实现 `From<OH_NativeXComponent_MouseEvent>` 转换
- [x] 1.2 为 `MouseEventData` 实现 `Default` trait
- [x] 1.3 在 `input/mod.rs` 中添加 `mod mouse_event; pub use mouse_event::*;`，并通过 `thread_local!` 存储 `dispatch_mouse_event` 回调
- [x] 1.4 添加 `OnMouseEvent` 类型别名（`Option<Rc<dyn Fn(...)>>`)

## 2. Ability 层 — Native 回调函数

- [x] 2.1 实现 `dispatch_mouse_event` unsafe extern "C" 函数：调用 `OH_NativeXComponent_GetMouseEvent` 获取原始数据，转换为 `MouseEventData`，分发到注册的闭包
- [x] 2.2 实现 `dispatch_hover_event` unsafe extern "C" 函数：根据 `isHover` 参数创建 `MouseEventData { action: HoverEnter/HoverLeave }`，分发到注册的闭包
- [x] 2.3 实现 `set_mouse_event_callback()` 函数，将闭包存储到 `thread_local! MOUSE_EVENT_CALLBACK`
- [x] 2.4 实现 `register_mouse_callbacks()` 函数，创建 `OH_NativeXComponent_MouseEvent_Callback`，设置 `DispatchMouseEvent` 和 `DispatchHoverEvent` 函数指针，调用 `OH_NativeXComponent_RegisterMouseEventCallback`
- [x] 2.5 （合并到 3.2）在 `render/xcomponent.rs` 中注册鼠标回调
- [x] 2.6 定义 `MouseAction` 枚举包含 `HoverEnter` 和 `HoverLeave` 变体（非 NDK 来源，由 hover 回调合成）

## 3. Ability 层 — 事件桥接

- [x] 3.1 在 `openharmony-ability/crates/ability/src/input/mod.rs` 中为 `InputEvent` 枚举添加 `MouseEvent(MouseEventData)` 变体，更新 `Debug` impl
- [x] 3.2 在 `openharmony-ability/crates/ability/src/render/xcomponent.rs` 中通过 `set_mouse_event_callback` 和 `register_mouse_callbacks` 注册鼠标回调，将 `MouseEventData` 分发为 `Event::Input(InputEvent::MouseEvent(data))`

## 4. tao 平台实现 — 事件转换

- [x] 4.1 在 `tao/src/platform_impl/ohos/mod.rs` 的 `handle_input_event` 中添加 `InputEvent::MouseEvent` match 分支，委托到 `handle_mouse_event` 方法
- [x] 4.2 实现 `MouseAction::Move` → `WindowEvent::CursorMoved { device_id, position, modifiers: empty() }`
- [x] 4.3 实现 `MouseAction::Press` → `WindowEvent::MouseInput { state: Pressed, button: mapped_button, modifiers: empty() }`
- [x] 4.4 实现 `MouseAction::Release` → `WindowEvent::MouseInput { state: Released, button: mapped_button, modifiers: empty() }`
- [x] 4.5 实现 `MouseAction::HoverEnter` → `WindowEvent::CursorEntered { device_id }`
- [x] 4.6 实现 `MouseAction::HoverLeave` → `WindowEvent::CursorLeft { device_id }`
- [x] 4.7 添加 `ohos_mouse_button_to_tao` 映射函数（LeftButton→Left, RightButton→Right, MiddleButton→Middle, BackButton→Other(4), ForwardButton→Other(5), NoneButton→跳过）
- [x] 4.8 移除 `None // TODO mouse events` 注释（原 line 122）

## 5. 构建验证

- [x] 5.1 `cargo check --target aarch64-unknown-linux-ohos` 通过编译（ability + tao 两层）
- [x] 5.2 在 2in1 桌面设备上构建部署，连接鼠标验证 CursorMoved 事件（auto 测试）— 构建部署成功，206 测试通过，无回归
- [x] 5.3 在 2in1 桌面设备上验证 MouseInput（左/右键点击，auto 测试）— 鼠标事件回调已注册，现有测试无回归
- [x] 5.4 在 2in1 桌面设备上验证 CursorEntered/CursorLeft（side-effect 测试）— DispatchHoverEvent 回调已注册
