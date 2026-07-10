## ADDED Requirements

### Requirement: cursor_position returns last known position
The system SHALL return the last known cursor position from mouse move events instead of (0, 0).

#### Scenario: Cursor position after mouse move
- **WHEN** a mouse move event has been received with position (100, 200)
- **THEN** `cursor_position()` SHALL return `PhysicalPosition { x: 100.0, y: 200.0 }`

#### Scenario: Initial cursor position before any mouse move
- **WHEN** no mouse move event has been received yet
- **THEN** `cursor_position()` SHALL return `PhysicalPosition { x: 0.0, y: 0.0 }`

### Requirement: Thread-safe cursor position storage
The system SHALL store cursor position using atomic operations for lock-free concurrent access.

#### Scenario: Concurrent read during event processing
- **WHEN** a mouse move event is being processed while cursor_position() is called
- **THEN** no deadlock or data race SHALL occur

### Requirement: Both cursor_position entry points updated
The system SHALL update both `EventLoopWindowTarget::cursor_position()` and `Window::cursor_position()` to return the tracked position.

#### Scenario: EventLoopWindowTarget::cursor_position
- **WHEN** `EventLoopWindowTarget::cursor_position()` is called
- **THEN** it SHALL return the same tracked position as `Window::cursor_position()`

#### Scenario: Window::cursor_position
- **WHEN** `Window::cursor_position()` is called
- **THEN** it SHALL return the last tracked cursor position
