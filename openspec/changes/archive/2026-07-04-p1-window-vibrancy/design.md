## Context

Tauri 的 `vibrancy` 模块为窗口提供视觉效果（模糊、毛玻璃、Mica 等），目前在 Windows 和 macOS 上有完整实现，OHOS 上为空操作。

**当前架构模式（Windows/macOS）**：
```
tauri/vibrancy/windows.rs → window_vibrancy::apply_blur(window, color)
  → windows-sys (DWM/SetWindowCompositionAttribute)   ← 平台原生 SDK

tauri/vibrancy/macos.rs → window_vibrancy::apply_vibrancy(window, material, ...)
  → objc2-app-kit (NSVisualEffectView)                 ← 平台原生 SDK
```

**OHOS 适配目标**：保持相同架构模式：
```
tauri/vibrancy/ohos.rs → window_vibrancy::apply_ohos_blur(window_id, radius)
  → openharmony-ability (NAPI → ArkTS backdropBlur)    ← 平台原生 SDK
```

**本地 SDK 验证**（HarmonyOS 6.1.0, API 23）：
- ❌ `Window.setWindowBlur()` 在 `@ohos.window` 中不存在（0 次出现）
- ✅ `backdropBlur(radius: number)` — 组件属性，API 7+
- ✅ `backgroundBlurStyle(BlurStyle)` — 组件属性，API 9+
- ✅ `NODE_BACKGROUND_BLUR_STYLE` — 原生节点 API（C/C++）

**约束**：
- 铁律 1：所有 OHOS 系统能力必须通过 `openharmony-ability` 桥接
- OHOS 模糊是组件级 API，不是窗口级 API
- `HasWindowHandle` 在 OHOS 返回 `OHNativeWindow`（渲染表面），无法用于操作 ArkUI 组件节点

## Goals / Non-Goals

**Goals:**
- 在 OHOS 上实现窗口模糊效果，让 `WindowEffectsConfig` 配置生效
- 保持 window-vibrancy 作为 tauri 和平台 SDK 之间的抽象层（与 Windows/macOS 一致）
- 支持 Tauri 的 Blur/Acrylic/Mica/Tabbed 等 Effect 类型
- 不支持的设备静默跳过

**Non-Goals:**
- 不实现 macOS 独有的材质类型的精确映射
- 不修改 `raw-window-handle` crate

## Decisions

### 决策 1：window-vibrancy 作为 OHOS 的抽象层

**选择**：`window-vibrancy` crate 新增 OHOS 平台支持，内部依赖 `openharmony-ability`（与 Windows 依赖 `windows-sys`、macOS 依赖 `objc2-app-kit` 模式一致）。

**新增 OHOS 专用 API**（不走 `HasWindowHandle`）：
```rust
// window-vibrancy/src/lib.rs 新增
#[cfg(target_env = "ohos")]
pub fn apply_ohos_blur(window_id: i64, radius: f64) -> Result<(), Error>;
#[cfg(target_env = "ohos")]
pub fn clear_ohos_blur(window_id: i64) -> Result<(), Error>;
#[cfg(target_env = "ohos")]
pub fn apply_ohos_acrylic(window_id: i64, radius: f64, color: Option<Color>) -> Result<(), Error>;
#[cfg(target_env = "ohos")]
pub fn clear_ohos_acrylic(window_id: i64) -> Result<(), Error>;
#[cfg(target_env = "ohos")]
pub fn apply_ohos_mica(window_id: i64, radius: f64, dark: Option<bool>) -> Result<(), Error>;
#[cfg(target_env = "ohos")]
pub fn clear_ohos_mica(window_id: i64) -> Result<(), Error>;
```

**理由**：
- 保持 window-vibrancy 作为平台抽象层的角色
- `HasWindowHandle` 在 OHOS 上返回 `OHNativeWindow`（渲染表面），无法操作 ArkUI 组件节点
- OHOS 模糊是组件级 API，需要不同的入口标识

**替代方案**：
- 修改 `HasWindowHandle` 签名 → 影响上游 `raw-window-handle` crate，不可行
- tauri 直接调用 openharmony-ability → 破坏三层架构一致性

### 决策 2：组件级模糊实现方式

**选择**：通过 `openharmony-ability` 的 ArkTS `WindowManager` 将 `backdropBlur(radius)` 应用到 WebView 容器组件。

**调用链**：
```
tauri/vibrancy/ohos.rs
  → window_vibrancy::apply_ohos_blur(window_id, radius)
    → openharmony_ability::set_window_blur(window_id, radius)
      → NAPI → ArkTS WindowManager.setWindowBlur(windowId, radius)
        → WebView 容器组件 .backdropBlur(radius) 动态更新
```

**动态更新机制**：见下方"实现演进（2026-07-07）"——`@State` 在 `@Builder`/BuilderNode 内不可用，实际用 `AttributeUpdater`（BlurModifier）运行时刷新 `backdropBlur`/`backgroundColor`。

**理由**：
- 组件级 API 是本地 SDK 中唯一可用的模糊方案
- `backdropBlur(radius: number)` API 7+，与项目最低版本兼容
- 运行时刷新用 `AttributeUpdater`（`modifier.attribute?.backdropBlur(radius)` 立即触发组件更新，不需 @State）

### 决策 3：Effect 到 OHOS 的映射策略

**选择**：所有 Effect 类型统一映射到 `apply_ohos_blur` + 可选的背景色设置

| Tauri Effect | window-vibrancy 调用 | 说明 |
|---|---|---|
| `Blur` | `apply_ohos_blur(id, radius)` | radius 取 config.radius 或默认 20 |
| `Acrylic` | `apply_ohos_acrylic(id, 25, color)` | 模糊 + 半透明背景色 |
| `Mica` | `apply_ohos_mica(id, 20, None)` | 中等模糊 |
| `MicaDark` | `apply_ohos_mica(id, 20, Some(true))` | 模糊 + 深色背景 |
| `MicaLight` | `apply_ohos_mica(id, 20, Some(false))` | 模糊 + 浅色背景 |
| `Tabbed` 系列 | 同 Mica 系列 | OHOS 无对应概念 |
| macOS 材质类 | `apply_ohos_blur(id, 20)` | 统一模糊近似 |

### 决策 4：通过 dispatcher 消息链传递窗口效果

**选择**：与 `set_background_color` 一致，通过 `WindowDispatch` trait → `WindowMessage` → event loop → tao Window 的消息链。

**理由**：tauri `Window<R>` 无法直接获取 OHOS window_id，需要经过 dispatcher 层在 event loop handler 中访问 tao Window 内部的 `window_id`。

**调用链**（设计初版，已简化）：
```
Window::set_effects(effects)
  → dispatcher.set_window_effects(effects)
    → WindowMessage::SetEffects(effects)
      → event loop handler
        → tao_window.set_window_effects(effects)
          → window_vibrancy::apply_ohos_blur(self.window_id, radius)
```

> **实现偏差（2026-07-02）**：上述 `SetEffects` → `tao_window.set_window_effects` 链路未采用。实际实现中，dispatcher 消息链仅用于取 window id（`WindowMessage::OhosWindowId` → `tao::WindowExtOpenHarmony::window_id()`），effect 应用由 `tauri/vibrancy/ohos.rs` 直接调用 `window_vibrancy::apply_ohos_blur(window_id, radius)`，不经 tao。这与 Windows/macOS 在 `tauri/vibrancy/mod.rs` 直接调用 `window_vibrancy` 的方式一致。详见 `openspec/changes/window-vibrancy-plan.md` 的架构决策。

## 实现演进（2026-07-07）：运行时刷新机制

初始实现（2026-07-04）只支持 build-time effects（`registerController` inject blurRadius 到 build data，`backdropBlur` 在 Stack build 时设置）。运行时 `setEffects` 因 `BuilderNode.update` 不刷新 `backdropBlur` 而失效。

### 运行时刷新：AttributeUpdater（BlurModifier）

`@Builder` 函数内不能用 `@State`，`BuilderNode.update` 不刷新组件属性。解决：`AttributeUpdater`（不需 @State，`attribute?.backdropBlur(radius)` 立即触发组件更新）。

```
// openharmony-ability DefaultWebview.ets
export class BlurModifier extends AttributeUpdater<CommonAttribute> {
  initializeModifier(_instance: CommonAttribute): void {
    // 空：让 build 时 Stack.backdropBlur(data.style.blurRadius) 生效，不覆盖
  }
}
// WebBuilder Stack: .backdropBlur(data.style.blurRadius).attributeModifier(data.blurModifier)
// 运行时: modifier.attribute?.backdropBlur(radius)  // 立即刷新
```

### build-time vs runtime 路径

| 路径 | 触发 | 机制 |
|---|---|---|
| build-time | `WindowBuilder::effects` → `build` 时 apply | `set_window_blur` → `applyWindowBlur` queue `pendingBlurs` → `registerController` inject blurRadius 到 build data → Stack `.backdropBlur(data.style.blurRadius)` build 时设置 |
| runtime | `Window::set_effects` → `run_on_main_thread` | `set_window_blur` → `applyWindowBlur` → `controller.setAllWebviewsBlurRadius` → `BlurModifier.attribute?.backdropBlur(radius)` 立即刷新 |

### TSFN（线程安全 NAPI，符合约束）

`set_window_blur` / `set_window_background_color` 用 TSFN（ThreadsafeFunction，fire-and-forget NonBlocking）。TSFN 线程安全，不需 `thread_local MAIN_THREAD_ENV`，任何线程可调。`Window::set_effects` + build-time apply 直接调（工作线程），不用 `run_on_main_thread`（符合 ohos-constraints.md 1.2 约束"禁止 run_on_main_thread + rx.recv()"）。

TSFN init 在 ArkHelper 初始化时（main thread，xcomponent.rs `init_vibrancy_tsfn`）。之后 set_window_blur 任何线程可调。

ohos_window_id 用 send_user_message + recv（不在 run_on_main_thread 闭包），工作线程调时 main thread event loop 处理 OhosWindowId，不死锁。

### set_window_background_color FnArgs

`set_window_background_color`（Acrylic/Mica tint）初始用裸 tuple `(i64, u32)` 调 NAPI `func.call`，导致只传 1 个参数（tuple 对象）而非 2 个，ArkTS 收到错误参数。修复：用 `FnArgs<(i64, u32)>`（与 `set_window_blur` 一致，7bd67be 修了 set_window_blur 但漏了 set_window_background_color）。

### 窗口创建：Float 子窗口

`WebviewWindow.new` 默认 `OHOSWindowKind::UIAbility`（singleton），与主窗口冲突（"UIAbility window already exists"）。vibrancy 测试窗口用 `create_transparent_window`（`WebviewWindowBuilder::new`，默认 `ohos_window_kind: None` → Float 子窗口），避开冲突。

## Risks / Trade-offs

- **[API 不存在]** `Window.setWindowBlur()` 在本地 SDK 中不存在 → 改用组件级 `backdropBlur`
- **[效果近似]** OHOS 无法精确复现 Windows Mica/Tabbed 的分层材质效果 → 文档标注为 "best-effort 近似"
- **[OHOS 专用 API 签名]** `apply_ohos_blur(window_id, radius)` 与 `apply_blur(window, color)` 签名不同 → OHOS 平台特殊性，无法避免
- **[文件数增加]** 需要修改 window-vibrancy + openharmony-ability + tauri 三个 crate（tao 仅经既有 trait 提供 window id，不新增代码）→ 每个改动都是模式化的平台适配
