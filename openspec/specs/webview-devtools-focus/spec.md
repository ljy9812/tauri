# webview-devtools-focus Specification

## Purpose
TBD - created by archiving change p2-webview-devtools-focus. Update Purpose after archive.
## Requirements
### Requirement: Enable web debugging access
The system SHALL enable OHOS web debugging by calling `WebviewController.setWebDebuggingAccess(true)` and recording the enabled state, when `open_devtools()` is invoked.

#### Scenario: open_devtools enables debugging
- **WHEN** `open_devtools()` is called
- **THEN** the system calls `WebviewController.setWebDebuggingAccess(true)` and the tracked debugging-access state becomes `true`

### Requirement: Disable web debugging access
The system SHALL disable OHOS web debugging by calling `WebviewController.setWebDebuggingAccess(false)` and recording the disabled state, when `close_devtools()` is invoked.

#### Scenario: close_devtools disables debugging
- **WHEN** `close_devtools()` is called
- **THEN** the system calls `WebviewController.setWebDebuggingAccess(false)` and the tracked debugging-access state becomes `false`

### Requirement: Report tracked debugging-access state
Because OHOS `WebviewController.setWebDebuggingAccess` has no getter, `is_devtools_open()` SHALL return the ArkTS-side tracked state (a module-level variable in `Utils.ets`, initialized from the webview `devtools` init flag and updated by `open_devtools`/`close_devtools`), NOT a query of the OHOS runtime. The ProxyJsHelper SHALL read this module variable directly (so the state is accurate even before the controller is bound, reflecting the init flag). It SHALL return `false` when neither the init `devtools=true` flag nor `open_devtools()` is in effect.

#### Scenario: is_devtools_open returns tracked state
- **WHEN** `open_devtools()` was called and `is_devtools_open()` is called
- **THEN** the system returns `true`

#### Scenario: is_devtools_open default false
- **WHEN** neither `open_devtools()` nor an init `devtools=true` flag is in effect
- **THEN** `is_devtools_open()` returns `false`

#### Scenario: is_devtools_open reflects init flag
- **WHEN** the webview was created with `devtools: true` (init flag) and `is_devtools_open()` is called
- **THEN** the system returns `true` (init called `setWebDebuggingAccess(true)` and set the tracked state)

#### Scenario: state persists across webview recreation
- **WHEN** a webview called `open_devtools()` (state true) and is then destroyed, and a new webview is created (with `devtools: false`)
- **THEN** `is_devtools_open()` on the new webview returns `true` (the module-level tracked state persists, matching the process-global sticky semantics of `setWebDebuggingAccess`)

### Requirement: DevTools methods keep cfg gate
`open_devtools`/`close_devtools`/`is_devtools_open` SHALL remain gated by `#[cfg(any(debug_assertions, feature = "devtools"))]`, consistent with the webview2 and wkwebview backends, so they only compile in debug or devtools-feature builds.

#### Scenario: devtools methods cfg-gated
- **WHEN** wry is built for OHOS in release without the `devtools` feature
- **THEN** `open_devtools`/`close_devtools`/`is_devtools_open` are not compiled (consistent with other platforms)

### Requirement: focus_parent focuses the webview
`focus_parent()` SHALL delegate to `openharmony-ability::helper::webview::Webview::focus()` (ArkTS `requestFocus`). On desktop platforms `focus_parent` focuses the **parent window** (webkitgtk `parent_window().focus()`, webview2 `SetFocus(parent HWND)`); OHOS has no separate parent window for the webview, so focusing the webview itself is the available approximation (not strictly equivalent for child-webview scenarios). `focus_parent` is currently a wry public API with no external caller (contract completion only). It SHALL NOT be cfg-gated.

#### Scenario: focus_parent calls requestFocus
- **WHEN** `focus_parent()` is called
- **THEN** the system invokes the ArkTS `requestFocus` on the webview and returns `Ok(())`

### Requirement: Global debugging-access semantics documented
Because `WebviewController.setWebDebuggingAccess` is a static global setting, `open_devtools`/`close_devtools` on one webview affect all webviews in the process. This global behavior SHALL be documented in code comments, distinguishing it from the per-webview DevTools behavior on desktop platforms.

#### Scenario: global effect noted
- **WHEN** the implementation toggles debugging access
- **THEN** a code comment notes that `setWebDebuggingAccess` is process-global on OHOS

