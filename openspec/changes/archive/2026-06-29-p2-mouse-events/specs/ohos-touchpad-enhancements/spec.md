## ADDED Requirements

### Requirement: Pinch scale extraction from axis events
The system SHALL extract the pinch scale value from ArkUI axis events using `OH_ArkUI_AxisEvent_GetPinchAxisScaleValue`.

#### Scenario: Touchpad two-finger pinch produces scale data
- **WHEN** a two-finger pinch gesture is performed on the touchpad
- **THEN** `AxisEventData.pinch_scale` SHALL contain the scale factor (1.0 = no change, >1.0 = zoom in, <1.0 = zoom out)

#### Scenario: No pinch produces zero scale
- **WHEN** a scroll event without pinch is received
- **THEN** `AxisEventData.pinch_scale` SHALL be 0.0

### Requirement: Input source type differentiation
The system SHALL identify the input source of axis events using `OH_ArkUI_UIInputEvent_GetSourceType`.

#### Scenario: Touchpad source identified
- **WHEN** an axis event originates from a touchpad
- **THEN** `AxisEventData.source_type` SHALL be `InputSourceType::Touchpad`

#### Scenario: Mouse source identified
- **WHEN** an axis event originates from a mouse wheel
- **THEN** `AxisEventData.source_type` SHALL be `InputSourceType::Mouse`

### Requirement: Touchpad scroll uses PixelDelta
The system SHALL use `MouseScrollDelta::PixelDelta` for touchpad scroll events and `MouseScrollDelta::LineDelta` for mouse wheel events.

#### Scenario: Touchpad two-finger scroll
- **WHEN** source_type is Touchpad and delta_x/delta_y is non-zero
- **THEN** the emitted `WindowEvent::MouseWheel` SHALL use `PixelDelta(PhysicalPosition { x, y })`

#### Scenario: Mouse wheel scroll
- **WHEN** source_type is Mouse and delta_y is non-zero
- **THEN** the emitted `WindowEvent::MouseWheel` SHALL use `LineDelta(x, y)`

### Requirement: Pinch scale triggers zoom via Ctrl+Wheel
The system SHALL emit a `MouseWheel` event with `ModifiersState::CONTROL` when pinch_scale indicates zoom.

#### Scenario: Pinch zoom in
- **WHEN** pinch_scale > 1.0
- **THEN** `WindowEvent::MouseWheel { delta: LineDelta(0, 1), modifiers: CONTROL }` SHALL be emitted

#### Scenario: Pinch zoom out
- **WHEN** pinch_scale < 1.0 and pinch_scale != 0.0
- **THEN** `WindowEvent::MouseWheel { delta: LineDelta(0, -1), modifiers: CONTROL }` SHALL be emitted

### Requirement: Manual test tracks scroll and pinch-zoom distinctly
The mouse tracking manual test SHALL differentiate scroll events from pinch-zoom events.

#### Scenario: Scroll event display
- **WHEN** a wheel event without ctrlKey is received
- **THEN** the event log SHALL display `scroll Δx=... Δy=...`

#### Scenario: Pinch-zoom event display
- **WHEN** a wheel event with ctrlKey is received
- **THEN** the event log SHALL display `pinch-zoom (0,...) btn=ctrl` in purple bold
