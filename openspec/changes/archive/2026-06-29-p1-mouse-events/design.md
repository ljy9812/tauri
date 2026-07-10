## Context

Tauri OHOS 平台当前仅处理触摸事件（TouchEvent）和键盘事件（KeyEvent），鼠标事件完全未实现（`tao/src/platform_impl/ohos/mod.rs:122` 标注 `TODO mouse events`）。OHOS NDK 自 API 9 起提供完整的鼠标事件 FFI 支持，`ohos-xcomponent-sys` v0.0.2 已包含所有必要的结构体和函数声明，`ohos-xcomponent-binding` v0.2.0 已定义 `MouseEvent` 和 `MouseButton` 枚举但未接入回调链路。

当前事件链路（已工作）：
```
ArkTS XComponent → NDK dispatch_touch_event → binding TouchEventData
→ ability InputEvent::TouchEvent → tao WindowEvent::Touch
```

目标事件链路（待实现）：
```
ArkTS XComponent → NDK DispatchMouseEvent → binding MouseEventData
→ ability InputEvent::MouseEvent → tao WindowEvent::CursorMoved/MouseInput

ArkTS XComponent → NDK DispatchHoverEvent → binding HoverEventData
→ ability InputEvent::MouseEvent(Hover) → tao WindowEvent::CursorEntered/CursorLeft
```

## Goals / Non-Goals

**Goals:**
- 在 OHOS 2in1 桌面设备上实现鼠标移动（CursorMoved）、左/右/中键点击（MouseInput）、光标进入/离开（CursorEntered/CursorLeft）
- 复用现有事件链路模式（XComponent callback → InputEvent → WindowEvent），与触摸事件路径一致
- 利用已有的 FFI 定义（ohos-xcomponent-sys v0.0.2），无需升级 sys crate

**Non-Goals:**
- 鼠标滚轮（MouseWheel）— 当前 FFI 结构体无 scroll delta 字段，`OH_NativeXComponent_GetExtraMouseEventInfo` 需要 API 20+ 且 sys v0.0.2 未绑定。后续单独 Phase 处理
- 触摸板手势（TouchpadPressure / AxisMotion）— 不在本 Phase 范围
- ArkTS 侧 `onMouse` 事件 — 当前方案纯 NDK 层实现，不涉及 ArkTS 代码修改
- Windows/macOS 行为对齐验证 — 仅确保 tao 事件类型正确，不做跨平台行为一致性测试

## Decisions

### D1: MouseEventData 结构体设计

**决策**：创建与 `TouchEventData` 对称的 `MouseEventData` 结构体

```rust
pub struct MouseEventData {
    pub x: f32,
    pub y: f32,
    pub screen_x: f32,
    pub screen_y: f32,
    pub timestamp: i64,
    pub action: MouseEvent,      // Press / Release / Move / None
    pub button: MouseButton,     // Left / Right / Middle / Back / Forward / NoneButton
}
```

**理由**：
- 与 `TouchEventData` 的命名风格和字段布局保持一致
- 直接映射 `OH_NativeXComponent_MouseEvent` FFI 结构体的所有字段
- 字段使用 snake_case 符合 Rust 命名规范（FFI 结构体使用 camelCase）

**替代方案**：直接透传 FFI 结构体 → 排除，因为违反 binding 层的安全封装原则

### D2: DispatchHoverEvent 集成方式

**决策**：将 `DispatchHoverEvent` 的 `isHover: bool` 转换为 `MouseEventData { action: HoverEnter/HoverLeave }`，通过同一个 `InputEvent::MouseEvent` 通道传递

```rust
// 扩展 MouseEvent 枚举
pub enum MouseEvent {
    None,
    Press,
    Release,
    Move,
    HoverEnter,   // 来自 DispatchHoverEvent(isHover=true)
    HoverLeave,   // 来自 DispatchHoverEvent(isHover=false)
}
```

**理由**：
- 复用同一条事件通道（`InputEvent::MouseEvent`），减少新增变体
- tao 层只需 match 更多 action 值即可生成 CursorEntered/CursorLeft
- hover 回调签名不同于 mouse callback（无 window 参数、无 x/y 坐标），需要特殊处理

**替代方案**：
- 新建 `InputEvent::HoverEvent(bool)` → 排除，增加了不必要的枚举变体
- 在 tao 层维护全局 `is_hovered` 状态 → 排除，状态管理复杂且容易不一致

### D3: HoverEvent 无坐标的处理

**决策**：`DispatchHoverEvent` 回调不提供 x/y 坐标。CursorEntered 和 CursorLeft 事件在 tao 中也不需要坐标（`CursorEntered { device_id }` / `CursorLeft { device_id }`），因此直接传递 `MouseEventData { x: 0.0, y: 0.0, ... }`

**理由**：tao 的 CursorEntered/CursorLeft 事件类型只包含 `device_id`，不携带位置信息

### D4: ohos-xcomponent-binding 升级策略

**决策**：由于 binding 是外部 crate（crates.io v0.2.0），采用 **fork 到 openharmony-ability 仓库** 的方式

**理由**：
- 避免等待上游发版周期
- openharmony-ability 已有对 binding 的深度依赖，fork 维护成本低
- 后续可向上游提 PR，同时不影响当前开发进度

**替代方案**：
- 直接修改 binding 源码 + 发布新版本 → 排除，周期长
- 在 ability 层直接调用 FFI → 排除，重复 binding 层已有的 callback 基础设施

### D5: 线程安全

**决策**：鼠标回调与触摸回调使用相同的事件分发模式

```rust
// 在 dispatch_mouse_event native callback 中：
let data = MouseEventData::from(raw_mouse_event);
if let Some(ref mut h) = *app.event_loop.borrow_mut() {
    h(Event::Input(InputEvent::MouseEvent(data)))
}
```

**理由**：
- 与 `dispatch_touch_event` 完全一致的模式，已经过生产验证
- `RefCell<borrow_mut>()` 保证同一时刻只有一个回调在执行
- 不需要 TSFN（因为 XComponent NDK 回调已在正确的线程上下文中）

## Risks / Trade-offs

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| ohos-xcomponent-binding fork 后上游更新 | fork 版本落后于上游 | 定期 rebase；最终目标合入上游后删除 fork |
| 鼠标事件在 mobile 设备上无意义 | 空回调开销 | NDK 在无鼠标设备时不触发回调，零开销 |
| HoverEvent 回调签名不同（无 window 参数） | 无法获取 window 指针 | hover 事件使用默认 WindowId，与触摸事件一致 |
| 缺少 MouseWheel 支持 | 滚轮功能缺失 | 标记为 Non-Goal，后续 Phase 通过 API 20+ 或 ArkTS 补齐 |

## Open Questions

1. **ohos-xcomponent-binding fork 位置**：是放在 `openharmony-ability/crates/` 下作为 workspace member，还是作为独立 git 子模块？建议前者，与现有 crate 组织方式一致。
