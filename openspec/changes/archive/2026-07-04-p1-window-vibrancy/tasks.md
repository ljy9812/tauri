## 1. openharmony-ability ArkTS 层

- [x] 1.1 在 `WindowManager.ets` 中新增 `setWindowBlur(windowId: number, radius: number): void` 方法，将 `backdropBlur(radius)` 应用到指定窗口的 WebView 容器组件，通过 @State 或 LocalStorage 动态更新

## 2. openharmony-ability Rust NAPI 层

- [x] 2.1 在 `crates/ability/src/window/mod.rs` 中新增 `set_window_blur(window_id: i64, radius: f64) -> napi_ohos::Result<()>` 函数，复用 `get_helper()` + `get_main_thread_env()` 模式调用 ArkTS `setWindowBlur`

## 3. window-vibrancy OHOS 平台支持

- [x] 3.1 在 `window-vibrancy/Cargo.toml` 中添加 OHOS 依赖：`[target.'cfg(target_env = "ohos")'.dependencies] openharmony-ability = { path = "..." }`
- [x] 3.2 新建 `window-vibrancy/src/ohos.rs`，实现 `apply_ohos_blur` / `clear_ohos_blur` / `apply_ohos_acrylic` / `clear_ohos_acrylic` / `apply_ohos_mica` / `clear_ohos_mica`，内部调用 `openharmony_ability::set_window_blur` + `set_window_background_color`
- [x] 3.3 修改 `window-vibrancy/src/lib.rs`：添加 `#[cfg(target_env = "ohos")] mod ohos;` 和 `pub use ohos::*;`，在 Error 枚举中添加 OHOS 相关错误变体

## 4. tao OHOS 窗口效果支持

- [x] 4.1 ~~在 `tao/Cargo.toml` 的 OHOS 依赖中添加 `window-vibrancy`~~ → 撤销：effect 应用不经 tao（与 Windows/macOS 一致，由 tauri vibrancy 层直接调 window-vibrancy），tao 仅经既有 `WindowExtOpenHarmony::window_id()` 提供 window id
- [x] 4.2 在 `tauri-runtime/src/lib.rs` 的 `WindowDispatch` trait 中添加 `ohos_window_id()` 方法，在 `tauri-runtime-wry/src/lib.rs` 中添加 `WindowMessage::OhosWindowId` variant + handler + dispatcher 实现
- [x] 4.3 ~~在 `tao/src/platform_impl/ohos/mod.rs` 中实现 `set_window_effects`~~ → 简化：tao 已提供 `WindowExtOpenHarmony::window_id()`，由 tauri vibrancy 层直接调用 window-vibrancy

## 5. tauri-runtime dispatcher 扩展

- [x] 5.1 ~~在 `tauri-runtime/src/window.rs` 的 `WindowDispatch` trait 中添加 `set_window_effects(effects: WindowEffectsConfig)` 方法~~ → 简化为 `ohos_window_id()` (已在 4.2 实现)
- [x] 5.2 ~~在 `tauri-runtime-wry/src/lib.rs` 的 `WindowMessage` 枚举中添加 `SetEffects(WindowEffectsConfig)` variant~~ → 简化为 `OhosWindowId` (已在 4.2 实现)
- [x] 5.3 ~~在 `tauri-runtime-wry/src/lib.rs` 的 event loop handler 中添加 `WindowMessage::SetEffects` 处理分支~~ → 简化为 `OhosWindowId` handler (已在 4.2 实现)

## 6. tauri vibrancy OHOS 集成

- [x] 6.1 修改 `tauri/crates/tauri/src/vibrancy/mod.rs`：添加 `#[cfg(target_env = "ohos")] mod ohos;` 和对应的 `ohos::apply_effects` / `ohos::clear_effects` 调用分支
- [x] 6.2 新建 `tauri/crates/tauri/src/vibrancy/ohos.rs`，实现 `apply_effects` 函数：通过 dispatcher 获取 OHOS window_id，调用 `window_vibrancy::apply_ohos_blur` 等 API；实现 `clear_effects` 函数
