## ADDED Requirements

### Requirement: MouseEventData struct in binding layer
The system SHALL provide a `MouseEventData` struct in the `ohos-xcomponent-binding` crate that wraps the FFI `OH_NativeXComponent_MouseEvent` with safe Rust types and snake_case field names.

#### Scenario: Construct from FFI struct
- **WHEN** `OH_NativeXComponent_GetMouseEvent` returns a raw `OH_NativeXComponent_MouseEvent`
- **THEN** the binding layer SHALL convert it to `MouseEventData` with fields `x: f32`, `y: f32`, `screen_x: f32`, `screen_y: f32`, `timestamp: i64`, `action: MouseEvent`, `button: MouseButton`

#### Scenario: MouseEvent enum covers all NDK actions
- **WHEN** the NDK reports mouse actions 0-3
- **THEN** the `MouseEvent` enum SHALL map them to `None(0)`, `Press(1)`, `Release(2)`, `Move(3)`

#### Scenario: MouseButton enum covers all NDK buttons
- **WHEN** the NDK reports button values 0, 1, 2, 4, 8, 16
- **THEN** the `MouseButton` enum SHALL map them to `NoneButton(0)`, `LeftButton(1)`, `RightButton(2)`, `MiddleButton(4)`, `BackButton(8)`, `ForwardButton(16)`

### Requirement: Mouse event callback registration
The system SHALL provide an `on_mouse_event` method on the XComponent that registers a Rust closure to receive mouse events from the NDK.

#### Scenario: Register callback via on_mouse_event
- **WHEN** `xcomponent.on_mouse_event(callback)` is called
- **THEN** the binding layer SHALL store the callback in `XComponentCallbacks.dispatch_mouse_event`
- **THEN** during `register_callback()`, the binding layer SHALL call `OH_NativeXComponent_RegisterMouseEventCallback` with a `DispatchMouseEvent` function pointer

#### Scenario: Callback receives MouseEventData
- **WHEN** the NDK fires `DispatchMouseEvent`
- **THEN** the binding layer SHALL call `OH_NativeXComponent_GetMouseEvent` to obtain raw data
- **THEN** convert it to `MouseEventData`
- **THEN** invoke the registered closure with `(XComponentRaw, WindowRaw, MouseEventData)`

### Requirement: InputEvent::MouseEvent variant in ability layer
The system SHALL add a `MouseEvent(MouseEventData)` variant to the `InputEvent` enum in openharmony-ability.

#### Scenario: Mouse event dispatched through InputEvent
- **WHEN** the binding layer's mouse event callback fires
- **THEN** the ability layer SHALL dispatch `Event::Input(InputEvent::MouseEvent(data))` through the event loop handler
- **THEN** the event SHALL reach tao's `handle_input_event` function

### Requirement: tao WindowEvent::CursorMoved from mouse move
The system SHALL convert `MouseEvent::Move` to tao's `WindowEvent::CursorMoved` event.

#### Scenario: Mouse move generates CursorMoved
- **WHEN** `handle_input_event` receives `InputEvent::MouseEvent` with `action == MouseEvent::Move`
- **THEN** tao SHALL emit `WindowEvent::CursorMoved { device_id, position: PhysicalPosition { x, y }, modifiers: ModifiersState::empty() }`

### Requirement: tao WindowEvent::MouseInput from press/release
The system SHALL convert `MouseEvent::Press` and `MouseEvent::Release` to tao's `WindowEvent::MouseInput` events.

#### Scenario: Left button press
- **WHEN** `InputEvent::MouseEvent` with `action == Press` and `button == LeftButton`
- **THEN** tao SHALL emit `WindowEvent::MouseInput { device_id, state: Pressed, button: MouseButton::Left, modifiers: empty() }`

#### Scenario: Right button release
- **WHEN** `InputEvent::MouseEvent` with `action == Release` and `button == RightButton`
- **THEN** tao SHALL emit `WindowEvent::MouseInput { device_id, state: Released, button: MouseButton::Right, modifiers: empty() }`

#### Scenario: Middle button press
- **WHEN** `InputEvent::MouseEvent` with `action == Press` and `button == MiddleButton`
- **THEN** tao SHALL emit `WindowEvent::MouseInput { device_id, state: Pressed, button: MouseButton::Middle, modifiers: empty() }`

### Requirement: MouseButton mapping from OHOS to tao
The system SHALL map OHOS `MouseButton` enum values to tao's `event::MouseButton` enum values.

#### Scenario: All button types mapped
- **WHEN** OHOS reports `LeftButton`, `RightButton`, `MiddleButton`, `BackButton`, `ForwardButton`
- **THEN** tao SHALL map to `event::MouseButton::Left`, `Right`, `Middle`, `Other(4)`, `Other(5)` respectively
- **THEN** `NoneButton` SHALL be ignored (no event emitted)

### Requirement: Device ID consistency
The system SHALL use a consistent `DeviceId` for all mouse events within a session.

#### Scenario: Same device ID across events
- **WHEN** multiple mouse events are received
- **THEN** all `CursorMoved` and `MouseInput` events SHALL use `DeviceId(0)` (matching the touch event device_id pattern)

### Requirement: MouseWheel stub
The system SHALL NOT emit `WindowEvent::MouseWheel` events in this Phase.

#### Scenario: No scroll events
- **WHEN** a mouse scroll action is performed
- **THEN** the system SHALL NOT emit any `MouseWheel` event
- **THEN** a debug log entry MAY indicate that mouse wheel is not yet supported
