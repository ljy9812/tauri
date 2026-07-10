## Why

R87 (New Window Create) is currently non-functional on OHOS: `NewWindowResponse::Create` is compiled out at the tauri-runtime level via `cfg(not(... target_env = "ohos"))`, and the wry OHOS bridge downgrades `Create` to `Allow` (dead code). The `Allow` path opens an in-page dialog (`promptAction.openCustomDialog`) rather than a real OS window. This means `window.open()` with `Create` semantics — creating a real OS sub-window — is impossible on OHOS, breaking parity with desktop platforms.

## What Changes

- **tauri-runtime** (`crates/tauri-runtime/src/webview.rs`): Remove `target_env = "ohos"` from the `cfg` gate on `NewWindowResponse::Create { window_id }` and the `WindowId` import, making the `Create` variant available on OHOS.
- **tauri-runtime-wry** (`crates/tauri-runtime-wry/src/lib.rs`): Add an OHOS-specific `Create` match arm that constructs fieldless `wry::NewWindowResponse::Create { }` (no webview lookup — OHOS cannot inject a webview instance).
- **wry OHOS** (`wry/src/ohos/mod.rs`): `Create` no longer degrades to `Allow`. Instead returns `OnWindowNewResult { allow: true, window_kind: "window" }` to signal ArkTS to create a real OS sub-window.
- **openharmony-ability Rust** (`crates/ability/src/helper/webview.rs`, `crates/ability/src/webview/mod.rs`): 
  - Extend `OnWindowNewResult` with `window_kind: Option<String>` field (`"dialog"` / `"window"` / `None`).
  - Change `on_window_new` handler return type from `bool` to `OnWindowNewResult`.
  - Add `generate_window_id() -> i64` NAPI function for ArkTS to obtain unique window IDs.
- **openharmony-ability ArkTS** (`native_ability/src/main/ets/webview/DefaultWebview.ets`, `native_ability/src/main/ets/ability/type.ets`):
  - `handleWindowNew` routes `window_kind == "window"` to `WindowManager.createSubWindow` + `loadUrl` instead of `openNewWindowDialog`.
  - `setWebController(newCtrl)` still called synchronously (ArkWeb contract), OS window creation deferred via `setTimeout`.

## Capabilities

### New Capabilities
- `webview-new-window-create`: Enables `NewWindowResponse::Create` on OHOS to create a real OS sub-window (via `WindowManager.createSubWindow`) instead of an in-page dialog. Distinct from `Allow` (dialog) and `Deny` (blocked).

### Modified Capabilities
None. This is a new capability that was previously unavailable on OHOS.

## Impact

- **4 repos modified**: tauri (tauri-runtime + tauri-runtime-wry), wry, openharmony-ability
- **tauri-runtime**: `NewWindowResponse::Create` and `WindowId` become available on OHOS — affects enum exhaustiveness in all match sites (need to verify no non-exhaustive matches exist)
- **tauri-runtime-wry**: New OHOS `Create` arm — does not affect desktop platforms (gated `cfg(target_env = "ohos")`)
- **wry**: `Create` variant on OHOS remains fieldless — no public API change for non-OHOS platforms
- **openharmony-ability**: `OnWindowNewResult` gains a field — ArkTS code reading `result.allow` remains backward compatible (new field is `Option`)
- **ArkTS**: `handleWindowNew` gains a branch for `window_kind == "window"` — existing `Allow`/`Deny` paths unchanged
- **Window management**: `WindowManager.createSubWindow` is called from a new code path — verify no reentrancy or race conditions with the existing `createOSWindow` flow
