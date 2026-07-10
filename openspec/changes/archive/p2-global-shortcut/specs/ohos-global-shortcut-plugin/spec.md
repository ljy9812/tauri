## ADDED Requirements

### Requirement: OHOS conditional compilation
The plugin SHALL use `cfg(target_env = "ohos")` to exclude the `global-hotkey` crate dependency on OHOS and instead depend on `openharmony-ability` with the `global_shortcut` feature. The `#![cfg]` gate at the top of `lib.rs` SHALL allow OHOS compilation.

#### Scenario: Compilation on OHOS
- **WHEN** the plugin is compiled with `target_env = "ohos"`
- **THEN** the `global-hotkey` crate is not included, and `openharmony-ability` (feature `global_shortcut`) is used instead

#### Scenario: Compilation on desktop
- **WHEN** the plugin is compiled without `target_env = "ohos"`
- **THEN** the `global-hotkey` crate is used as before, with no behavioral changes

### Requirement: OHOS shortcut registration via openharmony-ability
The plugin SHALL register shortcuts on OHOS by calling `openharmony_ability::register_shortcut(modifiers, key, id)` instead of `GlobalHotKeyManager::register()`. The shortcut string parsing (`"CmdOrCtrl+Shift+A"`) SHALL work identically on OHOS.

#### Scenario: Register shortcut on OHOS
- **WHEN** `register("Ctrl+Shift+X")` is called on OHOS
- **THEN** the string is parsed into modifiers `[Control, Shift]` and key `X`, and `openharmony_ability::register_shortcut()` is called with the assigned ID

#### Scenario: Register shortcut with unsupported modifier count
- **WHEN** `register("Ctrl+Shift+Alt+X")` is called on OHOS (3 modifiers)
- **THEN** the function returns an error indicating OHOS supports at most 2 modifiers

### Requirement: OHOS shortcut event handling
The plugin SHALL listen for shortcut events on OHOS via `openharmony_ability::shortcut_event_receiver()`. A background thread SHALL receive events and dispatch them to user handlers on the main thread. Both Pressed and Released events SHALL be forwarded.

#### Scenario: Shortcut triggered fires handler
- **WHEN** a registered shortcut is triggered on OHOS and `ShortcutEvent { id, state: Pressed }` is received
- **THEN** the corresponding handler is called with the shortcut and `ShortcutState::Pressed`

### Requirement: OHOS shortcut unregistration
The plugin SHALL unregister shortcuts on OHOS by calling `openharmony_ability::unregister_shortcut(id)`. `unregister_all()` SHALL call `openharmony_ability::unregister_all_shortcuts()`.

#### Scenario: Unregister shortcut on OHOS
- **WHEN** `unregister("Ctrl+Shift+X")` is called on OHOS
- **THEN** `openharmony_ability::unregister_shortcut(id)` is called for the matching ID

### Requirement: CLI BUILTIN_PLUGINS registration
The tauri-cli's `BUILTIN_PLUGINS` constant SHALL include `("global-shortcut", "@tauri/plugin-global-shortcut", "GlobalShortcutPlugin")` for OHOS plugin template generation.

#### Scenario: CLI recognizes global-shortcut for OHOS
- **WHEN** the tauri CLI generates OHOS plugin templates
- **THEN** `global-shortcut` is included in the built-in plugins list

### Requirement: Example app integration
The `examples/api` app SHALL include `tauri-plugin-global-shortcut` as an OHOS dependency and register it via `tauri_plugin_global_shortcut::Builder::new().build()` in the OHOS plugin setup block.

#### Scenario: Example app registers global-shortcut on OHOS
- **WHEN** the example app starts on OHOS
- **THEN** the global-shortcut plugin is registered and available for IPC commands

### Requirement: build.rs ohos_path
The plugin's `build.rs` SHALL call `.ohos_path("openharmony")` to register the OHOS native code directory.

#### Scenario: Build system recognizes OHOS path
- **WHEN** the plugin is compiled for OHOS
- **THEN** the build system includes the `openharmony/` directory in the OHOS build output
