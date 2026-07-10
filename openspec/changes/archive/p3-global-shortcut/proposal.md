## Why

Phase 1 和 Phase 2 已完成 openharmony-ability 桥接层和插件集成层。现在需要为 `tauri_plugin_global_shortcut` 的前端 API（`register`/`unregister`/`isRegistered`/`unregisterAll`）设计并实现设备端测试用例，验证 OHOS 上的行为与桌面平台一致。

## What Changes

- **src/lib/tests/plugins.ts**（或新建 `global-shortcut.ts`）：添加 global-shortcut 测试用例
- **capabilities 配置**：添加 `global-shortcut:allow-*` 权限
- **TestRunner.svelte**：添加 manual 测试按钮（物理键盘验证快捷键触发）

## Capabilities

### New Capabilities
- `ohos-global-shortcut-testing`: global-shortcut 插件前端测试用例，包括 auto（API 返回值验证）、side-effect（注册/注销状态变更）、manual（物理键盘快捷键触发验证）

### Modified Capabilities

（无）

## Impact

- **examples/api/src/lib/tests/**：新增 ~6 个测试用例
- **examples/api/src-tauri/capabilities/**：添加权限配置
- **examples/api/src/views/TestRunner.svelte**：添加 manual 测试按钮
- **不影响其他平台**：测试用例使用动态 `import()` 按需加载
