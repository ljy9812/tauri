## Context

`tauri_plugin_global_shortcut` 依赖 `global-hotkey 0.7` crate 实现跨平台全局快捷键，该 crate 在 Windows 上使用 `RegisterHotKey` Win32 API，macOS 上使用 Carbon `RegisterEventHotKey`，Linux 上使用 X11 `XGrabKey`。但 `global-hotkey` 不支持 OHOS。

OHOS 提供了 `@ohos.multimodalInput.inputConsumer` 模块（API 14+），支持通过 `on('hotkeyChange')` 订阅全局组合键事件。该 API 通过 ArkTS 调用，需要 NAPI 桥接才能从 Rust 侧使用。

openharmony-ability 是 Tauri OHOS 适配的唯一 ArkTS 桥接仓（铁律 #1），所有 OHOS 系统能力调用必须经过它。当前 openharmony-ability 已有类似的 TSFN 桥接模式（autostart、clipboard、updater）和键盘快捷键匹配器（`accelerator_matcher.ets`），可作为参考。

**约束**：
- tauri api demo 默认 API 版本为 12，`inputConsumer` 需要 API 14+，需要版本守卫
- `preKeys` 修饰键数量限制 [1, 2]，超过 2 个修饰键的快捷键无法注册
- Wearable 设备不支持（error 801）
- 禁止 `run_on_main_thread + rx.recv()` 阻塞模式（线程模型约束 #1.2）

## Goals / Non-Goals

**Goals:**
- 在 openharmony-ability 中实现完整的 global_shortcut 桥接模块
- 提供与 `global-hotkey` crate 对等的 Rust 公共 API（register/unregister/event listener）
- 通过 TSFN 实现 Rust → ArkTS 的注册/注销通道
- 通过 NAPI 回调 + crossbeam channel 实现 ArkTS → Rust 的事件通道
- API 14+ 版本守卫，低版本静默跳过

**Non-Goals:**
- 不修改 `global-hotkey` crate 本身
- 不支持 OHOS Wearable 设备
- 不支持超过 2 个修饰键的快捷键（OHOS API 限制）
- 不实现 `getAllSystemHotkeys()` 查询（不需要）
- 不实现 `on('keyPressed')` 单键监听（不在本 Phase 范围）
- Phase 1 不做插件集成（Phase 2 的内容）

## Decisions

### D1: 使用 TSFN Pattern C（crossbeam channel + TSFN forwarder）

**选择**：注册/注销请求通过 crossbeam channel 发送到 forwarder 线程，forwarder 调用 TSFN 触发 ArkTS 执行。

**理由**：
- 与 openharmony-ability 中 menu/statusbar 模块的模式一致
- 避免 `run_on_main_thread + recv()` 死锁风险（约束 #1.2）
- 允许从任意线程调用注册/注销

**替代方案**：
- Pattern B（直接 TSFN call_with_return_value）：更简单，但需要 async/await，不适合当前插件的同步 API
- Pattern D（主线程直接调用）：限制调用线程，不够灵活

### D2: 事件回调使用 NAPI `#[napi]` 函数 + crossbeam channel

**选择**：ArkTS 快捷键触发时调用 `emit_shortcut_event(id: u32)` NAPI 函数，该函数将事件推入 crossbeam channel，消费者通过 `shortcut_event_receiver()` 获取。

**理由**：
- 与 menu 的 `emit_menu_event()` 模式完全一致
- crossbeam channel 天然支持跨线程、多生产者/消费者
- 非阻塞，不会触发死锁

### D3: 键码映射使用字符串名称而非数字常量

**选择**：Rust 侧传递快捷键信息时使用 JSON 格式，包含键名（如 `"Ctrl"`, `"A"`, `"F5"`），ArkTS 侧使用 `KeyCode` 枚举转换为数字。

**理由**：
- 避免在 Rust 侧硬编码 OHOS 键值数字（不同 API 版本可能变化）
- ArkTS 侧可直接使用 `KeyCode.KEYCODE_A` 等常量，更可靠
- JSON 格式便于调试和日志

**替代方案**：
- 使用数字键值：更紧凑，但耦合特定 API 版本的键值定义

### D4: feature gate `global_shortcut`

**选择**：使用 Cargo feature `global_shortcut` 控制模块编译。

**理由**：
- 与 openharmony-ability 中 `menu`、`clipboard`、`statusbar` 等模块的 feature gate 模式一致
- 不需要快捷键功能的应用可以排除此模块，减小二进制体积

### D5: 版本守卫策略 — 静默跳过

**选择**：Rust 侧 `register_shortcut()` 在 `sdk_api_version() < 14` 时直接返回 `Ok(())`，不注册任何快捷键。

**理由**：
- 遵守约束 #6.4 的默认降级策略：静默跳过
- 与 Windows/macOS 行为一致：不支持的功能直接跳过
- 上层插件可通过 `is_registered()` 检测是否真正注册成功

### D6: HotkeyOptions 的 isRepeat 默认为 false

**选择**：注册快捷键时 `isRepeat` 默认设为 `false`。

**理由**：
- 与 `global-hotkey` crate 的行为一致：只在按下时触发一次 `Pressed` 和一次 `Released`
- 避免重复事件导致的应用逻辑混乱

### D7: 模拟 Pressed/Released 双事件

**选择**：OHOS `inputConsumer.on('hotkeyChange')` 只在按键按下时触发（无释放事件）。ArkTS 回调中需同时发送 `Pressed` 和 `Released` 两个事件到 Rust 侧，模拟与 `global-hotkey` crate 一致的行为。`ShortcutEvent` 结构体包含 `id: u32` 和 `state: ShortcutState` 字段。

**理由**：
- 桌面平台 `global-hotkey` 为每次快捷键激活发送 Pressed + Released 两个事件
- Tauri 插件的 JS 侧 `ShortcutHandler` 依赖 `state` 字段区分按下和释放
- 不发送 Released 事件会导致依赖 Released 状态的应用逻辑无法触发

**实现方式**：
- ArkTS 回调中依次调用 `emitShortcutEvent(id, "Pressed")` 和 `emitShortcutEvent(id, "Released")`
- NAPI 函数签名改为 `emit_shortcut_event(id: u32, state: String)`
- Rust 侧 `ShortcutEvent { id: u32, state: ShortcutState }`，`ShortcutState` 为 `Pressed`/`Released` 枚举

## Risks / Trade-offs

**[R1] preKeys 最多 2 个修饰键** → 在 `register_shortcut()` 中校验修饰键数量，超过 2 个时返回错误。上层插件需要处理此错误。

**[R2] API 14+ 限制** → 在 API 12 设备上快捷键功能完全不可用。通过版本守卫 + 静默跳过处理，不崩溃。文档标注最低版本要求。

**[R3] 快捷键冲突** → OHOS 返回 error 4200002（系统占用）或 4200003（已被其他应用订阅）。Rust 侧将这些错误码映射为注册失败，上层插件可查询 `is_registered()` 确认。

**[R4] 修饰键只有 Left 变体** → OHOS `inputConsumer.preKeys` 使用 `KEY_CTRL_LEFT` 等键值，不区分左右。这与 `global-hotkey` 的 `Modifiers::CONTROL`（不区分左右）语义一致，无影响。

**[R5] ArkTS 侧回调生命周期** → `inputConsumer.on()` 注册的回调在应用退出时需要清理。通过 `unregister_all_shortcuts()` 在应用销毁时统一清理。
