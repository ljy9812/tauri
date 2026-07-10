# webview-new-window-create Specification

## Purpose
TBD - created by archiving change p6-webview-new-window-create. Update Purpose after archive.
## Requirements
### Requirement: NewWindowResponse::Create available on OHOS

The `NewWindowResponse::Create { window_id }` variant SHALL be available on OHOS (target_env = "ohos"). The `cfg` gate in `tauri-runtime/src/webview.rs` SHALL NOT exclude OHOS from the `Create` variant or the `WindowId` import.

#### Scenario: Create variant compiles on OHOS
- **WHEN** compiling tauri-runtime for `target_env = "ohos"`
- **THEN** `NewWindowResponse::Create { window_id: WindowId }` SHALL be a valid variant
- **AND** code matching on `NewWindowResponse` SHALL handle `Create` without non-exhaustive errors

#### Scenario: Create variant not modified on non-OHOS platforms
- **WHEN** compiling tauri-runtime for Windows/macOS/Linux
- **THEN** `Create { window_id }` behavior SHALL be unchanged (webview lookup + injection)

### Requirement: tauri-runtime-wry passes Create through on OHOS

tauri-runtime-wry SHALL include an OHOS-specific `Create` match arm that constructs `wry::NewWindowResponse::Create { }` (fieldless on OHOS) without performing webview lookup. This arm SHALL be gated with `cfg(target_env = "ohos")` and SHALL NOT affect desktop platforms.

#### Scenario: Create from tauri-runtime reaches wry on OHOS
- **WHEN** the Tauri user's `new_window_handler` returns `NewWindowResponse::Create { window_id }` on OHOS
- **THEN** tauri-runtime-wry SHALL construct `wry::NewWindowResponse::Create { }` and pass it to wry
- **AND** wry OHOS SHALL receive `Create` (not `Allow`)

#### Scenario: Desktop Create unaffected
- **WHEN** compiling tauri-runtime-wry for non-OHOS desktop
- **THEN** the existing `Create { window_id }` arm (webview lookup + `wry::Create { webview }`) SHALL be unchanged

### Requirement: wry OHOS distinguishes Create from Allow

wry OHOS SHALL map `NewWindowResponse::Create` to `OnWindowNewResult { allow: true, window_kind: Some("window") }` and `NewWindowResponse::Allow` to `OnWindowNewResult { allow: true, window_kind: None }`. The `Create` variant SHALL NOT be degraded to `Allow`.

#### Scenario: Create returns window_kind
- **WHEN** wry OHOS receives `NewWindowResponse::Create { }`
- **THEN** the `on_window_new` callback SHALL return `OnWindowNewResult { allow: true, window_kind: Some("window") }`

#### Scenario: Allow returns no window_kind
- **WHEN** wry OHOS receives `NewWindowResponse::Allow`
- **THEN** the `on_window_new` callback SHALL return `OnWindowNewResult { allow: true, window_kind: None }`

#### Scenario: Deny returns allow=false
- **WHEN** wry OHOS receives `NewWindowResponse::Deny`
- **THEN** the `on_window_new` callback SHALL return `OnWindowNewResult { allow: false, window_kind: None }`

### Requirement: OnWindowNewResult extended with window_kind

`OnWindowNewResult` SHALL include a `window_kind: Option<String>` field. When `window_kind` is `Some("window")`, ArkTS SHALL create a real OS sub-window. When `window_kind` is `None` or `Some("dialog")`, ArkTS SHALL use the existing in-page dialog. The `allow` field semantics SHALL remain unchanged.

#### Scenario: window_kind field present in Rust struct
- **WHEN** `OnWindowNewResult` is constructed in Rust
- **THEN** it SHALL have `allow: bool` and `window_kind: Option<String>` fields

#### Scenario: window_kind field present in ArkTS interface
- **WHEN** ArkTS reads `OnWindowNewResult`
- **THEN** `result.window_kind` SHALL be accessible as `string | undefined`

### Requirement: on_window_new handler returns OnWindowNewResult

The `on_window_new` handler closure type SHALL change from `Fn(String, bool, bool) -> bool` to `Fn(String, bool, bool) -> OnWindowNewResult`. The NAPI layer SHALL pass the `OnWindowNewResult` through to ArkTS without wrapping.

#### Scenario: Handler returns OnWindowNewResult
- **WHEN** the wry OHOS bridge closure is called
- **THEN** it SHALL return `OnWindowNewResult` (not `bool`)
- **AND** the NAPI function SHALL return this directly to ArkTS

### Requirement: handleWindowNew creates real OS window for Create

When `result.window_kind == "window"`, `handleWindowNew` SHALL:
1. Synchronously call `event.handler.setWebController(newCtrl)` (ArkWeb contract)
2. Defer OS window creation via `setTimeout`
3. In the deferred callback: generate a window ID, call `WindowManager.createSubWindow`, then call `WindowManager.loadUrl` with the target URL

When `result.window_kind` is `None` or `"dialog"`, `handleWindowNew` SHALL use the existing `openNewWindowDialog` path (unchanged).

#### Scenario: Create opens real OS window
- **WHEN** JavaScript calls `window.open(url)` and the Tauri handler returns `Create { window_id }`
- **THEN** a real OS sub-window SHALL be created via `WindowManager.createSubWindow`
- **AND** the target URL SHALL be loaded in the new window's webview
- **AND** `setWebController` SHALL be called synchronously (ArkWeb contract satisfied)

#### Scenario: Allow still opens dialog
- **WHEN** JavaScript calls `window.open(url)` and the Tauri handler returns `Allow`
- **THEN** the in-page dialog (`openNewWindowDialog`) SHALL open (unchanged behavior)

#### Scenario: Deny still blocks
- **WHEN** JavaScript calls `window.open(url)` and the Tauri handler returns `Deny`
- **THEN** `setWebController(null)` SHALL be called (unchanged behavior)

### Requirement: generate_window_id NAPI function

A `generate_window_id() -> i64` NAPI function SHALL be available for ArkTS to obtain unique window IDs. It SHALL use the same `NEXT_WINDOW_ID` atomic counter as `create_os_window`, ensuring no ID collision between Rust-created and ArkTS-created windows.

#### Scenario: ArkTS generates unique window ID
- **WHEN** ArkTS calls `generateWindowId()` before creating a sub-window
- **THEN** the returned ID SHALL be unique (no collision with Rust-generated IDs)
- **AND** the ID SHALL be suitable for passing to `WindowManager.createSubWindow`

### Requirement: Platform limitation — no webview injection

OHOS `Create` SHALL NOT inject a user-provided webview instance (unlike desktop where `Create { webview }` supplies a pre-built webview). This is a platform limitation: OHOS `onWindowNew` only provides `handler.setWebController(ctrl)`, which accepts a `WebviewController` — not a `Web` component instance. The `window_id` from `Create { window_id }` is not used for webview lookup on OHOS.

#### Scenario: Create on OHOS does not look up webview
- **WHEN** tauri-runtime-wry receives `Create { window_id }` on OHOS
- **THEN** it SHALL NOT attempt to look up a webview by `window_id`
- **AND** it SHALL construct fieldless `wry::Create { }`

#### Scenario: Create semantics differ from desktop
- **WHEN** documenting `Create` behavior
- **THEN** OHOS `Create` SHALL be described as "create real OS sub-window" (not "inject user webview")

