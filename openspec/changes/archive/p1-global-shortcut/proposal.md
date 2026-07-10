## Why

`tauri_plugin_global_shortcut` 是 Tauri v2 中注册全局快捷键的标准插件，目前仅支持 Windows/macOS/Linux。该插件完全依赖 `global-hotkey` crate 作为平台后端，而 `global-hotkey` 不支持 OHOS。为了让 OHOS 上的 Tauri 应用能够注册和监听全局快捷键（如 `Ctrl+Shift+X`），需要在 openharmony-ability 中新增 NAPI 桥接模块，利用 OHOS 原生的 `inputConsumer` API（API 14+）实现快捷键订阅/取消订阅/事件回调。

## What Changes

- **新增 openharmony-ability `global_shortcut` 模块**：提供 Rust 侧 `register_shortcut()`、`unregister_shortcut()`、`unregister_all_shortcuts()`、`shortcut_event_receiver()` 公共 API
- **新增 ArkTS 侧 `globalShortcut.ets` helper**：调用 `inputConsumer.on('hotkeyChange')` / `off('hotkeyChange')` 实现快捷键订阅和取消
- **新增 NAPI 回调函数** `emit_shortcut_event()`：ArkTS 快捷键触发时回调 Rust，通过 crossbeam channel 分发事件
- **新增 TSFN 桥接**：Rust 注册/注销请求通过 crossbeam channel + TSFN forwarder 调用 ArkTS helper 函数
- **新增键码映射**：`global-hotkey` 的 `Code`/`Modifiers` ↔ OHOS `KeyCode` 键值常量的转换

## Capabilities

### New Capabilities
- `ohos-global-shortcut-bridge`: openharmony-ability 中的全局快捷键 NAPI 桥接能力，包括 Rust 侧公共 API、TSFN 注册/注销通道、ArkTS 侧 inputConsumer 集成、快捷键事件回调通道

### Modified Capabilities

（无已有 capability 的 spec 级行为变更）

## Impact

- **openharmony-ability crate**：新增 `global_shortcut` feature gate 和对应模块（~4 个 Rust 文件 + ~1 个 ArkTS 文件）
- **Cargo.toml**：`openharmony-ability` 新增 `global_shortcut` feature
- **ArkHelper 接口**：`ArkHelper.ets` 新增 `registerHotkey` / `unregisterHotkey` / `unregisterAllHotkeys` 方法
- **TSFN 初始化**：`render/xcomponent.rs` 中新增 global_shortcut TSFN 的创建和初始化
- **依赖**：无新增外部 crate 依赖，仅使用已有的 `crossbeam-channel`、`napi-ohos`、`serde`/`serde_json`
- **OHOS API 版本**：要求 API 14+（`inputConsumer` 首批接口版本），需要版本守卫
- **设备限制**：Wearable 设备不支持（返回 error 801），其他设备（Phone/Tablet/PC/TV）正常支持
