## ADDED Requirements

### Requirement: DispatchHoverEvent callback registration
The system SHALL register a `DispatchHoverEvent` callback alongside the `DispatchMouseEvent` when `on_mouse_event` is called.

#### Scenario: Hover callback registered with mouse callback
- **WHEN** `xcomponent.on_mouse_event(callback)` is called
- **THEN** during `register_callback()`, the binding layer SHALL call `OH_NativeXComponent_RegisterMouseEventCallback` with BOTH `DispatchMouseEvent` and `DispatchHoverEvent` function pointers set

#### Scenario: Hover callback receives isHover boolean
- **WHEN** the NDK fires `DispatchHoverEvent(component, isHover)`
- **THEN** the binding layer SHALL invoke the registered closure with `MouseEventData { action: HoverEnter/HoverLeave, x: 0.0, y: 0.0, ... }`

### Requirement: HoverEnter maps to CursorEntered
The system SHALL convert a `DispatchHoverEvent(isHover=true)` to tao's `WindowEvent::CursorEntered`.

#### Scenario: Mouse enters window
- **WHEN** `DispatchHoverEvent` fires with `isHover == true`
- **THEN** tao SHALL emit `WindowEvent::CursorEntered { device_id: DeviceId(0) }`

### Requirement: HoverLeave maps to CursorLeft
The system SHALL convert a `DispatchHoverEvent(isHover=false)` to tao's `WindowEvent::CursorLeft`.

#### Scenario: Mouse leaves window
- **WHEN** `DispatchHoverEvent` fires with `isHover == false`
- **THEN** tao SHALL emit `WindowEvent::CursorLeft { device_id: DeviceId(0) }`

### Requirement: MouseEvent enum extended for hover
The binding layer's `MouseEvent` enum SHALL include `HoverEnter` and `HoverLeave` variants that are not from the NDK but are synthesized from `DispatchHoverEvent`.

#### Scenario: Hover variants available
- **WHEN** code matches on `MouseEvent`
- **THEN** `HoverEnter` and `HoverLeave` SHALL be valid variants alongside `None`, `Press`, `Release`, `Move`

### Requirement: Hover event device ID
The system SHALL use `DeviceId(0)` for hover events, consistent with mouse and touch events.

#### Scenario: Consistent device ID
- **WHEN** `CursorEntered` or `CursorLeft` is emitted
- **THEN** the `device_id` field SHALL be `DeviceId(0)`

### Requirement: Desktop-only relevance
Hover events SHALL only be meaningful on desktop (2in1) devices. On mobile devices, the NDK does not fire hover callbacks.

#### Scenario: No hover on mobile
- **WHEN** the app runs on a mobile device (phone/tablet)
- **THEN** no `CursorEntered` or `CursorLeft` events SHALL be emitted (NDK does not trigger the callback)
