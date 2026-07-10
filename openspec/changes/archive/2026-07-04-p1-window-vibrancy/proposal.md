## Why

Tauri 的窗口视觉效果（模糊、毛玻璃、Mica 等）目前在 OHOS 平台上是空操作。用户在 OHOS 设备上配置 `window_effects` 后看不到任何效果，与 Windows/macOS 体验严重不一致。OHOS 提供了组件级模糊 API（`backdropBlur` / `NODE_BACKGROUND_BLUR_STYLE`），可以实现背景模糊效果，现在需要补全这一能力。

## What Changes

- 在 `window-vibrancy` crate 中新增 OHOS 平台支持，提供 `apply_ohos_blur` / `clear_ohos_blur` 等 OHOS 专用 API，内部依赖 `openharmony-ability` 作为平台 SDK（与 Windows 依赖 `windows-sys`、macOS 依赖 `objc2-app-kit` 模式一致）
- 在 `openharmony-ability` 的 Rust NAPI 层新增 `set_window_blur(window_id, radius)` 函数，桥接 OHOS 组件级模糊 API
- 在 `openharmony-ability` 的 ArkTS `WindowManager` 中新增 `setWindowBlur()` 方法，将模糊效果应用到 WebView 容器组件
- 在 `tauri` 的 `vibrancy` 模块新增 OHOS 平台实现（`ohos.rs`），调用 `window_vibrancy::apply_ohos_blur()`，保持与 Windows/macOS 相同的调用模式

## Capabilities

### New Capabilities
- `ohos-window-blur`: OHOS 平台窗口模糊效果适配，通过 window-vibrancy → openharmony-ability → 组件级 backdropBlur 的调用链，将 Tauri WindowEffect 枚举映射到 OHOS 模糊 API

### Modified Capabilities
<!-- 无需修改现有 spec -->

## Impact

- **受影响代码**：
  - `window-vibrancy` — 新增 OHOS 平台支持（`ohos.rs`、Cargo.toml、lib.rs）
  - `openharmony-ability` — Rust NAPI 层 (`window/mod.rs`) + ArkTS 层 (`WindowManager.ets`)
  - `tauri` — vibrancy 模块 (`mod.rs`, 新增 `ohos.rs`) + Cargo.toml
- **API 影响**：`window-vibrancy` 新增 OHOS 专用公开 API（`apply_ohos_blur` 等）
- **依赖**：`window-vibrancy` 在 OHOS cfg 下依赖 `openharmony-ability`；`tauri` 在 OHOS 下依赖 `window-vibrancy`
- **OHOS API**：使用组件级 `backdropBlur(radius)`（API 7+）或原生节点 API `NODE_BACKGROUND_BLUR_STYLE`
- **注意**：本地 SDK（HarmonyOS 6.1.0, API 23）中 `Window.setWindowBlur()` 不存在，模糊效果只能通过组件级 API 实现
