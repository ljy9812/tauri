## Why

Phase 1 在 openharmony-ability 中实现了 `global_shortcut` 模块，提供了 `register_shortcut()`、`unregister_shortcut()`、`shortcut_event_receiver()` 等 Rust API。但这些 API 尚未被 `tauri_plugin_global_shortcut` 插件使用。本 Phase 修改插件，使其在 OHOS 上使用 openharmony-ability 的 shortcut API 替代 `global-hotkey` crate，同时集成到示例应用和 CLI 工具链中。

## What Changes

- **plugins/global-shortcut/Cargo.toml**：OHOS 上排除 `global-hotkey` 依赖，添加 `openharmony-ability` (feature = "global_shortcut") 依赖
- **plugins/global-shortcut/build.rs**：添加 `.ohos_path("openharmony")`
- **plugins/global-shortcut/src/lib.rs**：添加 `cfg(target_env = "ohos")` 门控，定义 OHOS stub 类型（`OhosShortcut`、`OhosCode`、`OhosModifiers`），替换 `GlobalHotKeyManager` 和事件处理逻辑
- **tauri/crates/tauri-cli/src/mobile/open_harmony/plugins.rs**：在 `BUILTIN_PLUGINS` 中注册 `global-shortcut`
- **tauri/examples/api/src-tauri/**：添加插件依赖和注册代码

## Capabilities

### New Capabilities
- `ohos-global-shortcut-plugin`: global-shortcut 插件的 OHOS 平台适配，包括 cfg 隔离、OHOS stub 类型、openharmony-ability 集成、示例应用集成

### Modified Capabilities

（无）

## Impact

- **plugins-workspace (global-shortcut)**：~3 个文件修改
- **tauri**：~3 个文件修改（CLI BUILTIN_PLUGINS + examples/api 集成）
- **不影响其他平台**：所有修改通过 `cfg(target_env = "ohos")` 隔离
- **openharmony-ability**：无额外修改（Phase 1 已完成）
