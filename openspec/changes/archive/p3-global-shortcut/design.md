## Context

Phase 2 已将 `tauri_plugin_global_shortcut` 集成到 examples/api 示例应用。插件提供 4 个前端 IPC 命令：`register`、`unregister`、`unregister_all`、`is_registered`。

examples/api 使用自定义测试框架（`test-runner.ts`），测试用例分三类：
- `auto`：全自动断言，5 秒超时
- `side-effect`：有副作用但可验证
- `manual`：需人工交互，空 `fn()` 体

当前 plugins.ts 中无 global-shortcut 测试用例。

## Goals / Non-Goals

**Goals:**
- 覆盖 global-shortcut 4 个 IPC 命令的基本功能测试
- auto 测试验证 API 返回值类型和状态
- manual 测试验证物理键盘快捷键触发回调
- 添加必要的 ACL 权限配置

**Non-Goals:**
- 不测试 Rust 侧内部实现（Phase 1 的单元测试已覆盖）
- 不测试 OHOS 特有行为（如 Wearable 不支持）
- 不做跨平台一致性对比测试

## Decisions

### D1: 测试文件位置 — plugins.ts 内联

**选择**：在 `src/lib/tests/plugins.ts` 中添加测试用例，不创建新文件。

**理由**：
- 与现有插件测试（autostart、clipboard、dialog）保持一致的组织方式
- 避免 TestRunner.svelte 中增加新 import
- global-shortcut 测试用例数量少（~6 个），不值得单独文件

### D2: 测试分类

| 测试 | 分类 | 理由 |
|------|------|------|
| register + isRegistered 返回 true | auto | 纯 API 调用，可自动断言 |
| unregister + isRegistered 返回 false | auto | 纯 API 调用 |
| unregisterAll 后 isRegistered 返回 false | auto | 纯 API 调用 |
| register 返回的 handler 被调用 | manual | 需要物理键盘按下快捷键 |
| 多次 register/unregister 无错误 | side-effect | 有状态变更 |

### D3: 测试快捷键选择

**选择**：使用 `CommandOrControl+Shift+T`（T for Test）。

**理由**：
- 不与系统快捷键冲突
- `CommandOrControl` 在 OHOS 上解析为 Ctrl
- OHOS 支持最多 2 个修饰键，这里用了 2 个（Ctrl + Shift）

### D4: 权限配置

**选择**：在 capabilities 中添加 `global-shortcut:allow-register`、`global-shortcut:allow-unregister`、`global-shortcut:allow-unregister-all`、`global-shortcut:allow-is-registered`。

## Risks / Trade-offs

**[R1] OHOS 设备可能无物理键盘** → manual 测试仅在有键盘的设备上验证。auto/side-effect 测试不依赖键盘。

**[R2] API 14+ 限制** → 在 API < 14 设备上，register 静默成功（返回 undefined），isRegistered 可能返回 false。测试需要处理此情况。
