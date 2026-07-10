## 1. 测试用例实现

- [x] 1.1 在 `src/lib/tests/plugins.ts` 中添加 auto 测试：register + isRegistered 返回 true
- [x] 1.2 添加 auto 测试：register + unregister + isRegistered 返回 false
- [x] 1.3 添加 auto 测试：register + unregisterAll + isRegistered 返回 false
- [x] 1.4 添加 side-effect 测试：3 次 register/unregister 循环无错误
- [x] 1.5 添加 manual 测试：注册快捷键 + 物理键盘触发回调

## 2. 权限配置

- [x] 2.1 在 capabilities 配置中添加 global-shortcut:allow-* 权限

## 3. TestRunner 手动测试按钮

- [x] 3.1 在 TestRunner.svelte 中添加 global-shortcut 手动测试按钮

## 4. 验证

- [x] 4.1 确认 auto 测试在桌面平台通过
- [x] 4.2 确认 OHOS 设备端测试可运行（或标记为待设备验证）
