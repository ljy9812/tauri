## Context

`tauri_plugin_global_shortcut` 是一个单文件插件（`lib.rs` ~437 行），完全依赖 `global-hotkey 0.7` crate。该 crate 提供了：
- `HotKey`（Shortcut）类型，含 `from_str()` 解析和 `.id()` 方法
- `Code` 枚举（键盘键码）
- `Modifiers` 位标志（Ctrl/Shift/Alt/Super）
- `GlobalHotKeyManager` 用于注册/注销
- `GlobalHotKeyEvent` 用于事件监听
- `HotKeyState`（Pressed/Released）

Phase 1 已在 openharmony-ability 中实现了 `register_shortcut(&[Modifier], Key, u32)` 和 `shortcut_event_receiver() -> Receiver<ShortcutEvent>` API。

本 Phase 需要将插件从 `global-hotkey` 切换到 openharmony-ability，同时保持公共 API 不变。

## Goals / Non-Goals

**Goals:**
- 插件在 OHOS 上使用 openharmony-ability 注册/注销/监听全局快捷键
- 保持插件公共 API 不变（`Builder::new().build()`、`register()`、`unregister()` 等 IPC 命令）
- 前端 JS API 不变（`register()`、`unregister()`、`isRegistered()`）
- 集成到 examples/api 示例应用
- 在 tauri-cli BUILTIN_PLUGINS 中注册

**Non-Goals:**
- 不修改 openharmony-ability（Phase 1 已完成）
- 不修改 global-hotkey crate
- 不做前端测试（Phase 3 的内容）

## Decisions

### D1: lib.rs 内联 cfg 门控 + OHOS stub 类型

**选择**：在 `lib.rs` 顶部通过 `cfg(target_env = "ohos")` 定义 `OhosShortcut`、`OhosCode`、`OhosModifiers` 等 stub 类型，替代 `global-hotkey` 的类型。其余代码通过 cfg 门控选择不同实现路径。

**理由**：
- 插件是单文件结构，创建单独的 `mobile.rs` 会导致大量代码重复
- stub 类型保持与 `global-hotkey` 类型相同的接口（`from_str()`、`.id()`、`.to_string()`），最小化 cfg 门控范围
- 与 `dialog` 插件的 `mobile.rs` 模式不同：dialog 的命令处理逻辑简单（request-response），而 global-shortcut 需要持续事件监听，更适合在 `lib.rs` 内联处理

**替代方案**：
- 创建 `ohos.rs` 模块：需要重复整个插件结构，维护成本高
- Fork global-hotkey 添加 OHOS 支持：`global-hotkey` 不在我们控制的仓库中

### D2: OHOS 事件监听使用 spawn 线程 + shortcut_event_receiver

**选择**：在 `build()` 的 `setup()` 中 spawn 一个线程，循环 `shortcut_event_receiver().recv()`，收到事件后通过 `AppHandle::run_on_main_thread` 分发到 handler。

**理由**：
- `shortcut_event_receiver()` 返回 crossbeam `Receiver`，`recv()` 是阻塞的，必须在非主线程调用
- 收到事件后需要调用用户 handler（可能涉及 UI），需要在主线程执行
- 这与桌面 `GlobalHotKeyEvent::set_event_handler()` 的回调模式等价

### D3: OhosShortcut 使用自增 ID

**选择**：`OhosShortcut` 内部使用 `AtomicU32` 自增 ID，`from_str()` 解析字符串并分配 ID。

**理由**：
- `global-hotkey` 的 `HotKey::id()` 返回 `u32`，是内部自增的
- openharmony-ability 的 `register_shortcut()` 接受 `u32` ID
- 自增 ID 简单可靠

### D4: 快捷键字符串解析

**选择**：OHOS stub 的 `OhosShortcut::from_str()` 解析格式与 `global-hotkey` 相同（`"CmdOrCtrl+Shift+A"`），内部转换为 `Vec<OhosModifiers>` + `OhosCode`。

**理由**：
- 保持用户 API 不变：`Builder::new().with_shortcut("CmdOrCtrl+Shift+X")` 在 OHOS 上同样可用
- 解析逻辑简单：按 `+` 分割，最后一个 token 是 key，其余是 modifiers

### D5: Cargo.toml 依赖守卫

**选择**：
```toml
[target."cfg(not(any(target_os = \"android\", target_os = \"ios\", target_env = \"ohos\")))".dependencies]
global-hotkey = { version = "0.7", features = ["serde"] }

[target.'cfg(target_env = "ohos")'.dependencies]
openharmony-ability = { path = "...", features = ["global_shortcut"] }
```

**理由**：
- OHOS 上不需要 `global-hotkey`（它不支持 OHOS 且编译会失败）
- OHOS 上需要 `openharmony-ability` 的 `global_shortcut` feature

## Risks / Trade-offs

**[R1] lib.rs cfg 门控复杂度** → 需要仔细隔离每个使用 `global-hotkey` 类型的代码段。预计需要 ~10 处 cfg 门控。

**[R2] openharmony-ability path 依赖** → plugins-workspace 的 Cargo.toml 中需要使用相对路径或 git 路径引用 openharmony-ability。需确认 workspace 结构。

**[R3] 快捷键字符串解析兼容性** → OHOS stub 的解析器可能与 `global-hotkey` 的解析器行为略有不同（如特殊键名）。需要确保常见格式兼容。
