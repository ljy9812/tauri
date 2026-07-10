## 1. 模块骨架和 Feature Gate

- [x] 1.1 在 `crates/ability/Cargo.toml` 中新增 `global_shortcut` feature，添加 `crossbeam-channel` 和 `serde`/`serde_json` 依赖（如尚未有）
- [x] 1.2 创建 `crates/ability/src/global_shortcut/types.rs`，定义 `Modifier` 枚举（Control/Shift/Alt/Super）、`Key` 枚举（A-Z/0-9/F1-F24/Space/Enter/Escape/Tab 等常用键）、`ShortcutState` 枚举（Pressed/Released）、`ShortcutEvent { id: u32, state: ShortcutState }` 结构体，实现 Serialize/Deserialize
- [x] 1.3 创建 `crates/ability/src/global_shortcut/event.rs`，定义 crossbeam channel 的 sender/receiver 对（`SHORTCUT_EVENT_CHANNEL`），提供 `shortcut_event_receiver() -> Receiver<ShortcutEvent>` 和内部 `emit_event(event: ShortcutEvent)` 函数
- [x] 1.4 创建 `crates/ability/src/global_shortcut/mod.rs`，声明子模块，提供 `register_shortcut()`、`unregister_shortcut()`、`unregister_all_shortcuts()` 公共 API，内部维护已注册快捷键的 HashMap

## 2. TSFN 桥接层

- [x] 2.1 在 `crates/ability/src/global_shortcut/mod.rs` 中实现 `REGISTER_HOTKEY_TSFN` 和 `UNREGISTER_HOTKEY_TSFN` 的 static LazyLock 存储，参照 autostart.rs 的 TSFN 模式
- [x] 2.2 实现 `create_register_hotkey_tsfn(env: &Env)` 函数，创建调用 `helper.registerHotkey(id, modifiers, key)` 的 TSFN，使用 `callee_handled::<false>()`
- [x] 2.3 实现 `create_unregister_hotkey_tsfn(env: &Env)` 函数，创建调用 `helper.unregisterHotkey(id)` 的 TSFN，使用 `callee_handled::<false>()`
- [x] 2.4 在 `render/xcomponent.rs` 的 TSFN 初始化序列中调用 `create_register_hotkey_tsfn()` 和 `create_unregister_hotkey_tsfn()`，使用 `#[cfg(feature = "global_shortcut")]` 条件编译

## 3. NAPI 回调函数

- [x] 3.1 在 `crates/ability/src/global_shortcut/event.rs` 中实现 `#[napi]` 函数 `emit_shortcut_event(id: u32, state: String)`，内部将 state 字符串解析为 `ShortcutState` 枚举，调用 `emit_event(ShortcutEvent { id, state })`
- [x] 3.2 确保 NAPI 函数名自动转为 camelCase（`emitShortcutEvent`），ArkTS 侧使用 camelCase 调用，参数顺序为 `(id, state)`

## 4. 公共 API 实现

- [x] 4.1 实现 `register_shortcut(modifiers, key, id)` 函数：校验修饰键数量 ≤ 2，版本守卫（`sdk_api_version() < 14` 时静默跳过），构造 JSON 请求（id + modifier names + key name），通过 crossbeam channel 发送到 TSFN forwarder
- [x] 4.2 实现 forwarder 线程：spawn 一个线程 recv channel 请求，获取 TSFN 并调用 `tsfn.call(request, NonBlocking)`
- [x] 4.3 实现 `unregister_shortcut(id)` 函数：通过 channel 发送注销请求到 forwarder
- [x] 4.4 实现 `unregister_all_shortcuts()` 函数：遍历已注册 HashMap，逐个发送注销请求
- [x] 4.5 在 `crates/ability/src/lib.rs` 中添加 `#[cfg(feature = "global_shortcut")] mod global_shortcut;` 和 `pub use global_shortcut::*;` re-export

## 5. ArkTS 侧 Helper

- [x] 5.1 创建 `native_ability/src/main/ets/helper/global_shortcut.ets`，实现 `registerHotkey(id, modifiers, key)` 函数：将 modifier/key 名称映射到 `KeyCode` 常量，构造 `HotkeyOptions { preKeys, finalKey, isRepeat: false }`，调用 `inputConsumer.on('hotkeyChange')`，在回调中依次调用 `emitShortcutEvent(id, "Pressed")` 和 `emitShortcutEvent(id, "Released")`，保存 options 和 callback 引用以供 off 使用
- [x] 5.2 实现 `unregisterHotkey(id)` 函数：查找已注册的 options 和 callback，调用 `inputConsumer.off('hotkeyChange')`
- [x] 5.3 实现 `unregisterAllHotkeys()` 函数：遍历所有已注册项，逐个 off
- [x] 5.4 在 `helper/index.ets` 中 re-export global_shortcut 模块的函数
- [x] 5.5 在 `ability/ArkHelper.ets` 的 `createArkHelper()` 中注册 `registerHotkey`、`unregisterHotkey`、`unregisterAllHotkeys` 方法
- [x] 5.6 在 `ability/type.ets` 的 `ArkHelper` 接口中添加新方法签名

## 6. 测试 Stub

- [x] 6.1 在 `crates/ability/src/global_shortcut/types.rs` 中添加单元测试：验证 Modifier/Key 枚举的序列化/反序列化正确性
- [x] 6.2 添加单元测试：验证修饰键数量校验逻辑（> 2 返回错误）
- [x] 6.3 确认 `cargo check --features global_shortcut` 编译通过（OHOS target）
