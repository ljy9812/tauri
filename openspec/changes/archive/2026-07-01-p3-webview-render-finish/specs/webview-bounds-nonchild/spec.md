## ADDED Requirements

### Requirement: tao propagates ContentRectChange as Resized
tao's OHOS `ContentRectChange` event SHALL be propagated as `WindowEvent::Resized(PhysicalSize)` to the event loop (replacing the previous TODO warn). This ensures tauri's resize handler fires on window resize/fullscreen and calls `webview.set_bounds()` with the new dimensions.

#### Scenario: fullscreen triggers Resized
- **WHEN** the window is maximized/fullscreen (OHOS `windowRectChange` fires)
- **THEN** tao emits `WindowEvent::Resized(PhysicalSize)` with the new dimensions, and tauri's resize handler calls `set_bounds` for each webview

### Requirement: WindowIdStore preserves main window mapping
`WindowIdStore::insert` SHALL use `or_insert` (not `insert`) to prevent child window creation from overwriting the main window's HashMap entry. On OHOS, `WindowId` is a ZST — all windows share the same key — so `insert` would overwrite the main window's mapping with the child's, causing resize events to map to the wrong window.

#### Scenario: child window created after main window
- **WHEN** a child window is created after the main window
- **THEN** the main window's `WindowId → tauri WindowId` mapping is preserved (not overwritten), and subsequent resize events still map to the main window

### Requirement: Non-child webview set_bounds calls ArkTS setBounds
`set_bounds()` SHALL call `self.webview.set_bounds(x, y, w, h)` for **both child and non-child (main) webviews**, not cache-only for non-child. The `bounds_cache` SHALL be updated in both cases. This works because tao propagates `ContentRectChange` as `Resized` (D1) and `WindowIdStore` uses `or_insert` (D2), ensuring `set_bounds` is called on every window resize with the correct window_id mapping.

#### Scenario: non-child set_bounds takes effect on resize
- **WHEN** the window is resized (fullscreen/windowed)
- **THEN** `set_bounds` is called with the new dimensions, ArkTS `setBounds` → `applyStyle` → `updateWebviewStyle` → `node.update` → Web component re-renders with new `data.style.width/height/position`

#### Scenario: fullscreen no black bars
- **WHEN** the app window is maximized/fullscreen
- **THEN** the Web content fills the entire window with no black bars on any side

#### Scenario: set_bounds round-trip
- **WHEN** `set_bounds(original_bounds)` is called after `bounds()`
- **THEN** the call succeeds (no error) and `bounds()` returns matching values

## MODIFIED Requirements

### Requirement: Transparent background — child window only
The transparent background support (archive `p1-webview-transparent`) is verified as implemented for **child windows only** (FloatPage independent floating windows): `ArkHelper.ets` sets `init.transparent=true`, `DefaultWebview.ets` uses `RenderMode.SYNC_RENDER`, `DefaultXComponent.ets` has defensive transparent containers, `set_background_color` dynamically updates via monkey-patch. **Main window window-level transparency is NOT implemented** (requires OHOS window API such as `setWindowBackgroundColor`). R74 remains ⚠️ (partial).

#### Scenario: child window transparent
- **WHEN** a child window (FloatPage) is created with `transparent: true`
- **THEN** the Web component uses `RenderMode.SYNC_RENDER` and the floating window is transparent

#### Scenario: main window not transparent
- **WHEN** the main window is created with `transparent: true`
- **THEN** the webview content uses `RenderMode.SYNC_RENDER` but the main window background remains opaque (OHOS window-level transparency not implemented)
