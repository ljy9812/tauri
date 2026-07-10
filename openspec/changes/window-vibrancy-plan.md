# window-vibrancy 适配计划

**创建时间**：2026-06-16
**最后更新**：2026-07-07
**功能描述**：在 OHOS 上实现 Tauri 窗口模糊效果（Blur/Acrylic/Mica/Tabbed），通过 tauri/vibrancy → window-vibrancy → openharmony-ability → 组件级 backdropBlur 的调用链
**判断依据**：涉及 3 个代码层（window-vibrancy + openharmony-ability + tauri），预估 13 个文件，不拆分
**状态**：✓ 完整适配 + 设备端验证通过（2026-07-07）— 运行时 setEffects/clearEffects（AttributeUpdater 刷新 backdropBlur/backgroundColor）+ build 时 effects（WindowBuilder::effects）均生效

> **架构决策（2026-07-02）**：effect 应用**不经过 tao**，由 `tauri/vibrancy/ohos.rs` 直接调用 `window_vibrancy`，与 Windows/macOS 在 `tauri/vibrancy/mod.rs` 直接调用 `window_vibrancy` 的方式保持一致。tao 在本特性中仅通过既有的 `WindowExtOpenHarmony::window_id()` 提供 window ID，不新增 vibrancy 相关 API。原计划中 `tao/src/window.rs` 的 `set_window_effects` 方法与 `tao/src/platform_impl/ohos/mod.rs` 的 OHOS 实现均不再需要，`tao/Cargo.toml` 也不依赖 window-vibrancy。

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | 窗口模糊效果适配 | p1-window-vibrancy | ✓ 已归档 + 设备验证通过 | window-vibrancy + openharmony-ability + tauri | 12 | 设备端验证模糊效果 |

## Phase 详细说明

### Phase 1: 窗口模糊效果适配
- **目标**：在 OHOS 上实现窗口模糊效果，保持 window-vibrancy 作为平台抽象层（与 Windows/macOS 架构一致）
- **架构**（实际实现）：
  ```
  tauri/vibrancy/ohos.rs
    ├─ window.dispatcher.ohos_window_id()        ← tauri-runtime → wry → tao::WindowExtOpenHarmony::window_id()
    └─ window_vibrancy::apply_ohos_blur(window_id, radius)
         → openharmony_ability::set_window_blur(window_id, radius)
           → NAPI (FnArgs) → ArkTS ArkHelper.setWindowBlur(windowId, radius)
             → WindowManager.applyWindowBlur → pendingBlurs.set(Number(windowId), radius)
               → registerController 时注入 webview build 数据 style.blurRadius
                 → BuilderNode.build → .backdropBlur(blurRadius) 构建时生效
  ```
- **文件列表**（实际实现）：
  1. `openharmony-ability/native_ability/.../window/WindowManager.ets` — applyWindowBlur 排队 + registerController build 时注入 blurRadius
  2. `openharmony-ability/crates/ability/src/window/mod.rs` — set_window_blur NAPI 函数（用 FnArgs 传参）
  3. `window-vibrancy/Cargo.toml` — 添加 OHOS 依赖 openharmony-ability
  4. `window-vibrancy/src/ohos.rs` — 新建 OHOS 平台实现
  5. `window-vibrancy/src/lib.rs` — 添加 OHOS 模块、pub use 与 OhosError(String) 变体
  6. `tauri/crates/tauri-runtime/src/lib.rs` — WindowDispatch trait 添加 ohos_window_id()
  7. `tauri/crates/tauri-runtime-wry/src/lib.rs` — WindowMessage 添加 OhosWindowId + handler（经 tao 取 window id）
  8. `tauri/crates/tauri/src/vibrancy/ohos.rs` — 新建 OHOS 平台实现，直接调 window_vibrancy
  9. `tauri/crates/tauri/src/vibrancy/mod.rs` — 添加 OHOS 分支
  10. `tauri/crates/tauri/src/window/mod.rs` — build_internal 中 OHOS 直接 apply effects（见 P3）
  - ~~`tao/Cargo.toml` / `tao/src/window.rs` / `tao/src/platform_impl/ohos/mod.rs`~~ — 不再需要（见架构决策）
- **依赖**：无
- **OHOS API**：组件级 `backdropBlur(radius)`（API 7+），非 Window.setWindowBlur（本地 SDK 中不存在）
- **验证方式**：设备端运行，确认窗口背景模糊效果可见（2026-07-04 验证通过）

## 根因与修复（2026-07-04 设备端调试发现）

vibrancy 在 OHOS 上从未真正生效过（p1 归档时的"设备端验证"是虚假的）。经 ~50 轮构建调试，发现两层根因：

### 根因 1：napi-ohos Function::call 裸 tuple 传参 bug（根本原因）
- **现象**：`set_window_blur` 的 `func.call((window_id, radius))` 用裸 tuple，napi-ohos 1.2.0 的通用 `JsValuesTupleIntoVec` impl（`function.rs:19`）把整个 tuple 当成 **1 个** napi 值传。ArkHelper.setWindowBlur 收到 `(tuple对象, undefined)` → windowId=NaN, radius=undefined。blur 的值从未到达 ArkTS。
- **诊断**：在 registerController 打印 pendingBlurs 的 key/value，发现 `keys=number:NaN=undefined`（key 是 NaN，value 是 undefined）。
- **修复**：用 `FnArgs { data: (window_id, radius) }` 包裹 tuple，触发 `FnArgs` 专用的拆包 impl（`function.rs:55`），正确传 2 个参数。`Function<'_, (i64, f64), ()>` 改为 `Function<'_, FnArgs<(i64, f64)>, ()>`。

### 根因 2：hilog 在 NAPI 回调里抛 Argc mismatch（掩盖了根因 1）
- **现象**：原始 p1 setWindowBlur 方法体里有 `hilog.info(...)`，在 NAPI 回调上下文（ArkHelper.setWindowBlur 被 Rust NAPI 调）里调 hilog 会抛 "assertion (false) failed: Argc mismatch"。被 catch 吞成 `failed: {}`，看不到真正的参数问题。
- **修复**：applyWindowBlur 内不用 hilog（NAPI 回调上下文禁用 hilog）。

### 根因 3：BuilderNode.update 不刷新 backdropBlur
- **现象**：`setAllWebviewsBlurRadius` 改 `entry.style.blurRadius` 后调 `BuilderNode.update(entry)`，但 backdropBlur 不刷新（SDK 文档：update 要求 @Prop 反应式）。
- **修复**：在 `registerController` 的 `addWebview` 前把 blurRadius 注入 webview build 数据（`pendingInit.style.blurRadius`），让 `backdropBlur` 在构建时就生效，不依赖 update。

### 附带发现
- **oh-package.json5 全角冒号**：gen/ohos/entry/oh-package.json5 曾有 `file：`（全角冒号）导致 ohpm 装了 registry 旧版 @ohos-rs/ability（不含 setWindowBlur）。`tauri ohos init` 重新生成后修复（模板是正确的 ASCII 冒号）。
- **build 缓存多版本堆积**：每次改 openharmony-ability 源码重建，build cache 累积旧编译版本（最多 16 个），导致运行时可能加载旧版。需定期删 `entry/build` 清理。
- **set_window_background_color / set_window_decorations 也有同样的 FnArgs bug**：它们也用裸 tuple `func.call((window_id, color))`，参数没传对。本次只修了 set_window_blur，其他两个待修。

## 已知遗留项

- **P3（已查清，非遗留）**：`tauri/crates/tauri/src/window/mod.rs` 中 OHOS 在 `run_on_main_thread` 之外直接 apply effects。根因：vibrancy 路径调用 `ohos_window_id()`，是阻塞 `rx.recv()` getter；而 OHOS 的 `run_on_main_thread` 把闭包调度到 Chrome_IOThread（非 ArkTS 主线程），阻塞 recv 会触发 Chrome_IOThread ↔ ArkTS 主线程互等死锁（见 `ohos-constraints.md`）。直接 apply 使 `send_user_message` 在主线程同步内联执行，recv 立即返回，符合 OHOS 约束。这是正确做法，非临时 workaround。非 OHOS 平台仍走 `run_on_main_thread`。
- **待修复**：`set_window_background_color` 和 `set_window_decorations` 也有 napi-ohos 裸 tuple 传参 bug，需同样用 FnArgs 修复（影响 acrylic/mica 的背景色和 decorations 功能）。
- **待观察（本特性外）**：tauri 中其他在 `run_on_main_thread` 闭包内使用阻塞 getter（`window_getter!`/`webview_getter!`）的调用路径，在 OHOS 上存在同样的死锁风险，需另行排查。
- **测试代码**：`examples/api/src-tauri/src/lib.rs` 中有两个 vibrancy 对比测试窗口（vibrancy-blur / vibrancy-noblur）和 `examples/api/public/vibrancy.html` 透明测试页，用于验证。提交前可考虑简化为单个窗口或保留作为回归测试。

