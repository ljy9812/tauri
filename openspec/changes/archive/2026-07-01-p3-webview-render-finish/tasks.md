## 1. tao OHOS 层

- [x] 1.1 `tao/src/platform_impl/ohos/mod.rs`：`ContentRectChange` handler 从 `warn!("TODO:...")` 改为传播 `WindowEvent::Resized(PhysicalSize::new(rect.width, rect.height))`
- [x] 1.2 注释说明：ContentRectChange 经 windowRectChange 触发，传播为 Resized 使 tauri resize handler 调 set_bounds

## 2. tauri-runtime-wry 层

- [x] 2.1 `crates/tauri-runtime-wry/src/lib.rs`：`WindowIdStore::insert` 改为 `entry(w).or_insert(id)`（OHOS ZST WindowId 防覆盖）
- [x] 2.2 注释说明：OHOS WindowId 是 ZST，or_insert 保留主窗口映射

## 3. wry OHOS 层

- [x] 3.1 `wry/src/ohos/mod.rs`：`set_bounds` 移除 `if !self.is_child { cache-only; return; }` 早返回，子与非子统一调 `self.webview.set_bounds(x,y,w,h)` + 缓存
- [x] 3.2 注释说明：三修复链（tao 传播 + or_insert + 移除 cache-only）使 set_bounds 在 resize 时正确调用

## 4. R74 透明背景核实

- [x] 4.1 核实 archive `p1-webview-transparent` 已落地 → **仅子窗口生效**（FloatPage），主窗口窗口级透明未实现 → R74 维持 ⚠️

## 5. 测试用例

- [x] 5.1 自动用例：`set_bounds_test` 命令（bounds() → set_bounds() → bounds() round-trip）+ core.ts test 53
- [x] 5.2 手动用例：manual_tests.md 7.4 "全屏无黑边"（T0，防护三修复链回归）

## 6. 编译验证

- [x] 6.1 `cargo check`（host 非 ohos，wry）通过
- [x] 6.2 `cargo check --target aarch64-unknown-linux-ohos`（OHOS desktop，wry + tao + tauri-runtime-wry）通过
- [x] 6.3 OHOS mobile 编译通过
- [x] 6.4 设备验证：set_bounds auto test ✅ + 全屏无黑边 ✅ + cookie 无回归 ✅

## 7. 文档

- [x] 7.1 proposal/design/specs/tasks 更新为实际 3 仓修复方案
- [x] 7.2 plan 文件 Phase 3 状态更新
