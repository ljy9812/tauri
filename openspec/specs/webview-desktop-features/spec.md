# webview-desktop-features Specification

## Purpose
TBD - created by archiving change p7-webview-desktop-features. Update Purpose after archive.
## Requirements
### Requirement: R77 Window set_focus for Float sub-windows

`tao` OHOS `set_focus` SHALL bring a Float sub-window to the front and focus it. For the main UIAbility window, `set_focus` SHALL be a no-op (focus is OS-managed via `onActive`/`onForeground`), consistent with iOS/Android behavior.

#### Scenario: Float sub-window set_focus
- **WHEN** `set_focus` is called on a Float sub-window
- **THEN** the window SHALL be raised to the top and focused via `openharmony-ability` window API
- **AND** `is_focused` SHALL return `true` after focus is gained

#### Scenario: Main window set_focus is no-op
- **WHEN** `set_focus` is called on the main UIAbility window
- **THEN** no error SHALL be returned
- **AND** focus SHALL remain OS-managed (no programmatic change)

### Requirement: R77 Window set_focusable

`tao` OHOS `set_focusable(bool)` SHALL set whether a Float sub-window can receive focus. For the main window, it SHALL be a no-op.

#### Scenario: Float sub-window set_focusable
- **WHEN** `set_focusable(false)` is called on a Float sub-window
- **THEN** the window SHALL not accept focus from subsequent `set_focus` calls or user interaction

### Requirement: R75 WebViewBuilderExtOhos with_https_scheme

A `with_https_scheme(self, enabled: bool) -> Self` method SHALL be added to `WebViewBuilderExtOhos`. When `enabled = true`, custom protocols SHALL use `https://` origin semantics. When `false` or not called, the default raw scheme behavior SHALL be preserved.

#### Scenario: HTTPS scheme enabled
- **WHEN** `.with_https_scheme(true)` is set on the WebViewBuilder
- **THEN** custom protocol requests SHALL have `https://` origin
- **AND** secure-context features (crypto.subtle, etc.) SHALL be accessible

#### Scenario: Default raw scheme preserved
- **WHEN** `.with_https_scheme` is not called
- **THEN** custom protocol behavior SHALL be unchanged (raw scheme)

### Requirement: R82 Clipboard attribute is always-on (platform limitation)

The wry `clipboard` WebViewAttribute SHALL be silently ignored on OHOS. ArkWeb allows page-level clipboard access by default. This is the same behavior as macOS.

#### Scenario: with_clipboard ignored
- **WHEN** `.with_clipboard(true)` or `.with_clipboard(false)` is called
- **THEN** clipboard access SHALL be always enabled on OHOS
- **AND** no error or warning SHALL be emitted

### Requirement: R85 Data directory intentionally unused (design decision)

The `data_directory` WebViewAttribute SHALL NOT be used on OHOS. The OHOS Web component automatically stores web data in the app sandbox. This is the same behavior as Android and is an intentional design decision.

#### Scenario: data_directory ignored
- **WHEN** `data_directory` is set in Tauri config
- **THEN** it SHALL be silently ignored on OHOS
- **AND** web data SHALL be stored in the app's sandbox directory

### Requirement: R86 PathResolver directories

The `PathResolver` OHOS implementation SHALL provide correct paths for all standard directories. The `base_path` SHALL be the app's el2 sandbox root, and directory joins SHALL NOT produce duplicated path segments.

#### Scenario: app_data_dir path correctness
- **WHEN** `app_data_dir()` is called
- **THEN** the returned path SHALL NOT contain duplicated `files/files` segments
- **AND** the path SHALL be a valid directory within the app sandbox

### Requirement: R90 Click-through not supported (platform limitation)

`set_ignore_cursor_events` SHALL return `Err(NotSupported)` on OHOS. OHOS does not provide a window-level click-through API.

#### Scenario: set_ignore_cursor_events returns error
- **WHEN** `set_ignore_cursor_events(true)` is called
- **THEN** `Err(ExternalError::NotSupported)` SHALL be returned
- **AND** no crash or side effect SHALL occur

### Requirement: R91 Hotkey zoom works on OHOS desktop

The JS-based hotkey zoom (`zoom-hotkey.js`) SHALL be injected on OHOS desktop (`cfg(desktop)`). The `set_webview_zoom` IPC command SHALL be registered. Programmatic `zoom()` SHALL work via `WebviewController.zoom()`.

#### Scenario: Hotkey zoom on desktop
- **WHEN** user presses Ctrl+`=` on OHOS desktop
- **THEN** the webview SHALL zoom in
- **AND** the zoom level SHALL be applied via `controller.zoom()`

#### Scenario: No hotkey zoom on mobile
- **WHEN** OHOS_DEVICE_TYPE is not desktop
- **THEN** `zoom-hotkey.js` SHALL NOT be injected
- **AND** `set_webview_zoom` command SHALL NOT be registered

