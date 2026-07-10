## ADDED Requirements

### Requirement: Register global shortcut
The system SHALL provide a `register_shortcut(modifiers: &[Modifier], key: Key, id: u32)` function that registers a global keyboard shortcut via OHOS `inputConsumer.on('hotkeyChange')`. The function SHALL send the shortcut definition (modifier names + key name) via crossbeam channel to a TSFN forwarder thread, which invokes the ArkTS `registerHotkey()` helper function. The function SHALL be callable from any thread.

#### Scenario: Successful registration with single modifier
- **WHEN** `register_shortcut(&[Modifier::Control], Key::A, 1)` is called on API 14+ device
- **THEN** the shortcut Ctrl+A is registered with `inputConsumer.on('hotkeyChange')` and the function returns `Ok(())`

#### Scenario: Registration with two modifiers
- **WHEN** `register_shortcut(&[Modifier::Control, Modifier::Shift], Key::X, 2)` is called
- **THEN** the shortcut Ctrl+Shift+X is registered with `preKeys: [KEY_CTRL_LEFT, KEY_SHIFT_LEFT]` and `finalKey: KEYCODE_X`

#### Scenario: More than 2 modifiers rejected
- **WHEN** `register_shortcut(&[Modifier::Control, Modifier::Shift, Modifier::Alt], Key::Z, 3)` is called
- **THEN** the function returns `Err` with a message indicating OHOS supports at most 2 modifier keys

#### Scenario: API version below 14
- **WHEN** `register_shortcut()` is called on a device with `sdk_api_version() < 14`
- **THEN** the function returns `Ok(())` without registering any shortcut (silent skip)

#### Scenario: Wearable device
- **WHEN** `register_shortcut()` is called on a Wearable device and OHOS returns error code 801
- **THEN** the function returns `Err` with a message indicating the device does not support global shortcuts

#### Scenario: System-occupied shortcut
- **WHEN** `register_shortcut()` is called for a shortcut already occupied by the system (OHOS error 4200002)
- **THEN** the function returns `Err` with a message indicating the shortcut is occupied by the system

### Requirement: Unregister global shortcut
The system SHALL provide an `unregister_shortcut(id: u32)` function that unregisters a previously registered shortcut via `inputConsumer.off('hotkeyChange')`. The function SHALL send the unregister request via crossbeam channel to the TSFN forwarder.

#### Scenario: Successful unregistration
- **WHEN** `unregister_shortcut(1)` is called for a previously registered shortcut
- **THEN** the shortcut is unregistered via `inputConsumer.off()` and the function returns `Ok(())`

#### Scenario: Unregister non-existent shortcut
- **WHEN** `unregister_shortcut(999)` is called for an ID that was never registered
- **THEN** the function returns `Ok(())` (idempotent, no error)

### Requirement: Unregister all shortcuts
The system SHALL provide an `unregister_all_shortcuts()` function that unregisters all shortcuts registered by this module. It SHALL iterate over all registered shortcuts and call `inputConsumer.off()` for each.

#### Scenario: Unregister all when shortcuts exist
- **WHEN** `unregister_all_shortcuts()` is called with 3 registered shortcuts
- **THEN** all 3 shortcuts are unregistered and the function returns `Ok(())`

#### Scenario: Unregister all when no shortcuts registered
- **WHEN** `unregister_all_shortcuts()` is called with 0 registered shortcuts
- **THEN** the function returns `Ok(())` immediately

### Requirement: Shortcut event receiver
The system SHALL provide a `shortcut_event_receiver() -> Receiver<ShortcutEvent>` function that returns a crossbeam channel receiver for shortcut trigger events. `ShortcutEvent` SHALL contain `id: u32` and `state: ShortcutState` (enum: `Pressed`/`Released`). When a registered shortcut is triggered on the OHOS device, ArkTS SHALL emit both `Pressed` and `Released` events sequentially (since OHOS `inputConsumer` only fires on key down, the Rust side simulates the dual-event behavior to match `global-hotkey` crate behavior on desktop).

#### Scenario: Shortcut triggered emits Pressed and Released
- **WHEN** the user presses Ctrl+A and shortcut ID 1 was registered for Ctrl+A
- **THEN** ArkTS calls `emitShortcutEvent(1, "Pressed")` then `emitShortcutEvent(1, "Released")` via NAPI, and the receiver yields `ShortcutEvent { id: 1, state: Pressed }` followed by `ShortcutEvent { id: 1, state: Released }`

#### Scenario: Multiple shortcuts triggered in sequence
- **WHEN** the user presses Ctrl+A (id=1) then Ctrl+Shift+X (id=2)
- **THEN** the receiver yields: Pressed(1), Released(1), Pressed(2), Released(2)

#### Scenario: Unregistered shortcut does not trigger event
- **WHEN** the user presses a key combination that matches a previously unregistered shortcut
- **THEN** no event is pushed to the channel

### Requirement: TSFN initialization
The system SHALL create the `registerHotkey` and `unregisterHotkey` TSFNs during `render()` in `render/xcomponent.rs`, following the same pattern as autostart TSFNs. The TSFNs SHALL use `callee_handled::<false>()` and be stored in static `LazyLock<RwLock<Option<Arc<Tsfn>>>>`.

#### Scenario: TSFN created during render
- **WHEN** the ability `render()` is called
- **THEN** `create_register_hotkey_tsfn()` and `create_unregister_hotkey_tsfn()` are called, and the TSFNs are stored in their respective static variables

### Requirement: NAPI callback for shortcut events
The system SHALL expose a `#[napi]` function `emit_shortcut_event(id: u32, state: String)` that ArkTS calls when a shortcut is triggered. This function SHALL push `ShortcutEvent { id, state }` onto the crossbeam channel.

#### Scenario: ArkTS calls emitShortcutEvent with Pressed state
- **WHEN** ArkTS calls `emitShortcutEvent(42, "Pressed")` (camelCase per napi-derive-ohos rule)
- **THEN** `ShortcutEvent { id: 42, state: Pressed }` is pushed to the event channel

#### Scenario: ArkTS calls emitShortcutEvent with Released state
- **WHEN** ArkTS calls `emitShortcutEvent(42, "Released")`
- **THEN** `ShortcutEvent { id: 42, state: Released }` is pushed to the event channel

### Requirement: ArkTS helper functions
The system SHALL provide ArkTS helper functions `registerHotkey(id, modifiers, key)` and `unregisterHotkey(id)` on the ArkHelper object. These functions SHALL call `inputConsumer.on('hotkeyChange')` and `inputConsumer.off('hotkeyChange')` respectively, using `KeyCode` constants from `@kit.InputKit`.

#### Scenario: registerHotkey called from TSFN
- **WHEN** TSFN invokes `helper.registerHotkey(1, ["Ctrl"], "A")`
- **THEN** ArkTS constructs `HotkeyOptions { preKeys: [KeyCode.KEYCODE_CTRL_LEFT], finalKey: KeyCode.KEYCODE_A, isRepeat: false }` and calls `inputConsumer.on('hotkeyChange', options, callback)`

#### Scenario: unregisterHotkey called from TSFN
- **WHEN** TSFN invokes `helper.unregisterHotkey(1)`
- **THEN** ArkTS calls `inputConsumer.off('hotkeyChange')` with the previously registered options and callback

#### Scenario: Shortcut triggered fires callback
- **WHEN** `inputConsumer` fires the hotkeyChange callback for a registered shortcut with id=1
- **THEN** ArkTS calls `emitShortcutEvent(1, "Pressed")` followed by `emitShortcutEvent(1, "Released")` via NAPI

### Requirement: Key code mapping
The system SHALL provide a key code mapping that converts Tauri/global-hotkey `Code` and `Modifiers` types to OHOS `KeyCode` constants. The mapping SHALL cover: all letter keys (A-Z), digit keys (0-9), function keys (F1-F24), common special keys (Space, Enter, Escape, Tab, etc.), and modifier keys (Control, Shift, Alt, Super/Meta).

#### Scenario: Letter key mapping
- **WHEN** the Rust side sends key name `"A"` to ArkTS
- **THEN** ArkTS maps it to `KeyCode.KEYCODE_A`

#### Scenario: Function key mapping
- **WHEN** the Rust side sends key name `"F5"` to ArkTS
- **THEN** ArkTS maps it to `KeyCode.KEYCODE_F5`

#### Scenario: Unknown key
- **WHEN** the Rust side sends an unrecognized key name
- **THEN** ArkTS logs a warning and the registration fails with an appropriate error
