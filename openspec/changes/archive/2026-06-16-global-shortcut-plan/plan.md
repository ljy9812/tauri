# Global Shortcut 适配计划

**创建时间**：2026-06-13
**功能描述**：适配 tauri_plugin_global_shortcut 到 OHOS 平台，支持 Builder::new().build() 及 register/unregister/isRegistered 等 API
**判断依据**：涉及 3 个代码层，预估 ~12 个文件

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | 底层实现 | p1-global-shortcut | ✓ 已实现 | openharmony-ability | 5-6 | cargo check + Rust UT stub |
| 2 | 上层集成 | p2-global-shortcut | ✓ 已实现 | plugins-workspace/global-shortcut + tauri | 5-6 | cargo check + 设备端构建 |
| 3 | 前端测试 | p3-global-shortcut | ✓ 已实现 | tauri examples/api | 2-3 | 设备端自动/手动测试 |
| 4 | 差距修复 | p4-global-shortcut | ✓ 无 gap | 审计发现的 gap | 按需 | 全链路验证 |

## Phase 详细说明

### Phase 1: 底层实现 — openharmony-ability global_shortcut 模块
- **目标**：在 openharmony-ability 中新增 `global_shortcut` 模块，提供 Rust 侧快捷键注册/注销/事件监听 API，通过 NAPI/TSFN 桥接到 ArkTS `inputConsumer` API
- **文件列表**：
  - `crates/ability/src/global_shortcut/mod.rs` — Rust 公共 API + TSFN 管理
  - `crates/ability/src/global_shortcut/types.rs` — 键码映射、Shortcut 数据结构
  - `crates/ability/src/global_shortcut/event.rs` — 快捷键事件通道（crossbeam）
  - `crates/ability/src/lib.rs` — 模块声明 + re-export
  - `crates/ability/Cargo.toml` — 添加 `global_shortcut` feature
  - `native_ability/src/main/ets/helper/global_shortcut.ets` — ArkTS 侧 inputConsumer 调用
  - `native_ability/src/main/ets/ability/ArkHelper.ets` — 注册 shortcut helper 函数
- **依赖**：无

### Phase 2: 上层集成 — plugins-workspace global-shortcut OHOS 适配
- **目标**：修改 global-shortcut 插件，在 OHOS 上使用 openharmony-ability 的 shortcut API 替代 global-hotkey crate
- **文件列表**：
  - `plugins/global-shortcut/Cargo.toml` — OHOS 条件依赖
  - `plugins/global-shortcut/build.rs` — 添加 ohos_path
  - `plugins/global-shortcut/src/lib.rs` — cfg 隔离
  - `plugins/global-shortcut/src/mobile.rs` — OHOS 平台实现
  - `tauri/crates/tauri-cli/src/mobile/open_harmony/plugins.rs` — BUILTIN_PLUGINS 注册
  - `tauri/examples/api/src-tauri/Cargo.toml` — 添加 global-shortcut 依赖
  - `tauri/examples/api/src-tauri/src/lib.rs` — 注册插件
- **依赖**：Phase 1 完成

### Phase 3: 前端测试 — 测试用例设计和集成
- **目标**：设计并实现前端 API 测试用例，在设备端验证 global-shortcut 功能
- **文件列表**：
  - `tauri/examples/api/src/lib/tests/global-shortcut/` — auto/side-effect/manual 测试
  - `tauri/examples/api/src/views/` — 测试 UI
  - `tauri/examples/api/src-tauri/capabilities/` — 权限配置
- **依赖**：Phase 2 完成

### Phase 4: 差距修复
- **目标**：审计 Phase 1-3 的产出，修复发现的 gap
- **依赖**：Phase 3 完成

## 技术要点

### OHOS API
- 模块：`@ohos.multimodalInput.inputConsumer`（API version 14+）
- 订阅：`inputConsumer.on('hotkeyChange', hotkeyOptions, callback)`
- 取消：`inputConsumer.off('hotkeyChange', hotkeyOptions, callback)`
- 数据结构：`HotkeyOptions { preKeys: Array<number>[1,2], finalKey: number, isRepeat: boolean }`
- 错误码：4200002（系统占用）、4200003（已被其他应用订阅）

### TSFN 模式
- 使用 Pattern C（crossbeam channel + TSFN forwarder）
- Rust → ArkTS：register/unregister 请求通过 crossbeam channel 发送，forwarder 线程 recv 后调用 TSFN
- ArkTS → Rust：快捷键触发事件通过 NAPI `emit_shortcut_event()` 回调

### 约束
- API 14+ 版本守卫
- preKeys 最多 2 个修饰键
- Wearable 设备不支持（error 801）
- 需要 `openharmony-ability` 作为唯一 ArkTS 桥接（铁律 #1）
- `cfg(target_env = "ohos")` 隔离（铁律 #2）
