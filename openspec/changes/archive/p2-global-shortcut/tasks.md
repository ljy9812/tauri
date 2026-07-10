## 1. Cargo.toml 和 build.rs 修改

- [x] 1.1 修改 `plugins/global-shortcut/Cargo.toml`：将 `global-hotkey` 依赖守卫添加 `target_env = "ohos"` 排除，添加 OHOS 条件依赖 `openharmony-ability`（feature = "global_shortcut"），添加 `openharmony` 到 platform support metadata
- [x] 1.2 修改 `plugins/global-shortcut/build.rs`：添加 `.ohos_path("openharmony")`

## 2. lib.rs cfg 隔离和 OHOS stub 类型

- [x] 2.1 在 `lib.rs` 顶部修改 `#![cfg]` gate 允许 OHOS 编译（排除 android/ios 但不排除 ohos）
- [x] 2.2 添加 `#[cfg(target_env = "ohos")]` OHOS stub 类型定义：`OhosModifiers`（枚举）、`OhosCode`（枚举，覆盖常用键）、`OhosShortcut`（含 id/from_str/to_string/modifiers/code 方法）
- [x] 2.3 添加 `#[cfg(not(target_env = "ohos"))]` 类型别名：`type OhosShortcut = Shortcut;` 等，使后续代码统一使用 `OhosShortcut` 名称
- [x] 2.4 修改 `GlobalShortcut` 结构体：OHOS 上移除 `GlobalHotKeyManager` 字段，仅保留 `AppHandle` 和 `shortcuts` HashMap
- [x] 2.5 修改 `build()` 的 `setup()` 闭包：OHOS 上不创建 `GlobalHotKeyManager`，不设置 `GlobalHotKeyEvent::set_event_handler()`；改为 spawn 线程监听 `shortcut_event_receiver()`
- [x] 2.6 修改 `register_internal()`：OHOS 上调用 `openharmony_ability::register_shortcut()` 而非 `manager.register()`
- [x] 2.7 修改 `unregister` 系列方法：OHOS 上调用 `openharmony_ability::unregister_shortcut()` / `unregister_all_shortcuts()`
- [x] 2.8 修改 `is_registered()`：OHOS 上通过内部 HashMap 查询（无 manager 依赖）

## 3. CLI 和示例应用集成

- [x] 3.1 在 `tauri/crates/tauri-cli/src/mobile/open_harmony/plugins.rs` 的 `BUILTIN_PLUGINS` 中添加 `("global-shortcut", "@tauri/plugin-global-shortcut", "GlobalShortcutPlugin")`
- [x] 3.2 在 `tauri/examples/api/src-tauri/Cargo.toml` 的 OHOS 依赖区域添加 `tauri-plugin-global-shortcut`
- [x] 3.3 在 `tauri/examples/api/src-tauri/src/lib.rs` 的 OHOS 插件注册区域添加 `tauri_plugin_global_shortcut::Builder::new().build()`
- [x] 3.4 确认 `plugins/global-shortcut` 的 `openharmony/` 目录存在（可以是空的占位目录，因为 OHOS ArkTS 代码在 openharmony-ability 中）

## 4. 编译验证

- [x] 4.1 确认桌面 `cargo check` 编译通过（不受 OHOS 修改影响）
- [x] 4.2 确认 OHOS target 编译通过（或标记为待设备端验证）
