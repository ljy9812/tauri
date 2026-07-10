## Context

OHOS Tauri 的 `NewWindowResponse::Create` 变体在 OHOS 平台上完全不可用：

1. **tauri-runtime** (`webview.rs:174`): `Create { window_id: WindowId }` 被 `cfg(not(any(target_os = "android", target_os = "ios", target_env = "ohos")))` 排除 — 变体不存在
2. **tauri-runtime-wry** (`lib.rs:5224`): `Create` match arm 被 `cfg(all(desktop, not(target_env = "ohos")))` 排除 — MingyuChen 的编译修复
3. **wry OHOS** (`mod.rs:225`): `Create` 降级为 `Allow`（死代码 — tauri-runtime-wry 不产生 `Create`）
4. **ArkTS**: `handleWindowNew` 只检查 `result.allow`（bool）— 无窗口类型区分

当前 `Allow` 路径通过 `promptAction.openCustomDialog` 创建页内浮层对话框（`NewWindowDialog.ets`），嵌入嵌套 `Web` 组件。这不是真实 OS 窗口。

现有基础设施：
- `WindowManager.createSubWindow(opts)` — 真实 OS 子窗口创建（`windowStage.createSubWindow`）
- `WindowManager.loadUrl(id, url)` — 在窗口中加载 URL
- `ArkHelper.createOSWindow(config)` — Rust → ArkTS 的窗口创建 NAPI 桥接
- 三级异步队列：`pendingInits` / `pendingJsHelperProxies` / `pendingUrls` — 处理 controller 未就绪竞态

## Goals / Non-Goals

**Goals:**
- 让 `NewWindowResponse::Create` 在 OHOS 上可用，创建真实 OS 子窗口（通过 `WindowManager.createSubWindow`）
- `Create` 与 `Allow` 行为区分：`Create` = 真窗口，`Allow` = 页内对话框
- 不影响现有 `Allow`/`Deny` 路径
- 不影响桌面平台（Windows/macOS/Linux）的 `Create` 实现

**Non-Goals:**
- 不实现 webview 实例注入（OHOS `onWindowNew` 只返回 `bool`，无法注入 webview — 桌面 `Create` 的语义是"用户提供 webview 实例"，OHOS 无法实现此语义）
- 不传播 `NewWindowFeatures`（size/position）— OHOS `onWindowNew` 不提供窗口特性，`NewWindowFeatures` 在 OHOS 上始终为 `None`
- 不修改 `Allow` 路径的对话框行为
- 不实现 `onWindowNewExt`（API 12+ 扩展事件，当前代码中不存在）

## Decisions

### D1: 在 tauri-runtime 层解除 `Create` 的 OHOS 排除

**决策**: 从 `cfg(not(any(target_os = "android", target_os = "ios", target_env = "ohos")))` 中移除 `target_env = "ohos"`，使 `Create { window_id: WindowId }` 在 OHOS 上可用。

**替代方案**: 不修改 tauri-runtime，仅在 wry OHOS 层处理 — 不可行，因为 tauri-runtime-wry 的 match 语句在 OHOS 上不编译 `Create` arm，wry OHOS 永远收不到 `Create`。

**WindowId 兼容性**: tao 的 OHOS `WindowId` 是 ZST（零大小类型），已存在。移除 `target_env = "ohos"` 后 `WindowId` import 可用。tauri-runtime-wry 的 `Create` arm 在 OHOS 上不通过 `window_id` 查找 webview（无法注入），所以 ZST 的 key 冲突问题不影响此路径。

### D2: tauri-runtime-wry 的 OHOS `Create` arm 构造无字段 `wry::Create { }`

**决策**: 在 tauri-runtime-wry 中，OHOS 上的 `Create` arm 不查找 webview、不构造 `wry::Create { webview }`（字段不存在），而是构造无字段的 `wry::NewWindowResponse::Create { }`。

**理由**: OHOS 无法注入 webview 实例。`window_id` 不传递到 wry 层 — wry OHOS 只需要知道"用户要求创建真窗口"，不需要知道 `window_id`。

```rust
// tauri-runtime-wry OHOS Create arm（新增）
#[cfg(target_env = "ohos")]
tauri_runtime::webview::NewWindowResponse::Create { .. } => {
    wry::NewWindowResponse::Create { }
}
```

### D3: wry OHOS `Create` 返回带 `window_kind` 的 `OnWindowNewResult`

**决策**: wry OHOS 桥接闭包的返回类型从 `bool` 改为 `OnWindowNewResult`。`Create` 返回 `{ allow: true, window_kind: Some("window") }`，`Allow` 返回 `{ allow: true, window_kind: None }`，`Deny` 返回 `{ allow: false, window_kind: None }`。

**替代方案**: 保持返回 `bool`，在 ArkTS 侧用额外参数区分 — 不可行，因为 `on_window_new` NAPI 回调的返回类型是 `OnWindowNewResult`，`bool` 需要包装。直接返回 `OnWindowNewResult` 更自然。

### D4: 扩展 `OnWindowNewResult` 增加 `window_kind` 字段

**决策**: `OnWindowNewResult` 增加 `window_kind: Option<String>` 字段。值为 `"window"`（真窗口）、`None`（对话框，默认）。

**向后兼容**: 新字段是 `Option`，ArkTS 侧 `result.window_kind` 未定义时走默认对话框路径。现有 `result.allow` 读取不受影响。

### D5: ArkTS `handleWindowNew` 路由到 `WindowManager.createSubWindow`

**决策**: 当 `result.window_kind == "window"` 时：
1. 同步调用 `event.handler.setWebController(newCtrl)` — 满足 ArkWeb 合约
2. `setTimeout(() => { ... }, 0)` 延迟创建 OS 窗口（避免阻塞渲染线程）
3. 在延迟回调中调用 `WindowManager.createSubWindow` + `loadUrl`

**窗口 ID 生成**: 新增 `generate_window_id() -> i64` NAPI 函数，复用 Rust 的 `NEXT_WINDOW_ID` 原子计数器，确保 ID 全局唯一。

**setWebController 合约**: `newCtrl` 被传递给 ArkWeb 但不用于新窗口（新窗口的 `DefaultXComponent` 创建自己的 controller）。`newCtrl` 实质上被丢弃 — 这是可接受的，因为 ArkWeb 合约只要求"调用 setWebController"，不要求 controller 被实际使用。

### D6: `on_window_new` handler 返回类型从 `bool` 改为 `OnWindowNewResult`

**决策**: openharmony-ability 的 `WebViewBuilder::on_window_new` 方法签名从 `F: Fn(String, bool, bool) -> bool` 改为 `F: Fn(String, bool, bool) -> OnWindowNewResult`。

**影响**: wry OHOS 是唯一调用方，更新 wry OHOS 桥接闭包即可。NAPI 层的 `create_function_from_closure` 返回 `OnWindowNewResult`（已经是返回类型），无需修改。

### D7: tauri 层 NewWindowResponse 枚举添加 Create 变体（审计新增）

**审计发现**: tauri crate (`webview/mod.rs:268-274`) 在 OHOS 上定义了独立的 `NewWindowResponse` 枚举，只有 `Allow(PhantomData<R>)` 和 `Deny`，**没有 `Create` 变体**。这意味着 Tauri 用户在 OHOS 上无法返回 `Create`。

**决策**: 在 OHOS 的 `NewWindowResponse` 枚举中添加 `Create { window: WebviewWindow<R> }` 变体，与非 OHOS 版本一致。保留 `Allow(PhantomData<R>)` 不变（向后兼容）。

**额外修改**: tauri 的 `webview/mod.rs:725-739` match 站点需要添加 OHOS `Create` arm，将 `WebviewWindow<R>` 的 window ID 映射到 `tauri_runtime::NewWindowResponse::Create { window_id }`。

**影响**: 这是 5 仓库修改（不是原计划的 4 仓库），增加了 tauri crate。

## Risks / Trade-offs

- **[Risk] `NewWindowResponse` match 非穷尽** → 移除 tauri-runtime 的 `Create` cfg 后，所有 match `NewWindowResponse` 的代码都需要处理 `Create` 变体。需全局搜索 match 站点，确保 OHOS 上不会出现非穷尽匹配编译错误。
- **[Risk] `WindowId` ZST 导致 `window_id` 查找错误** → tauri-runtime-wry 的 OHOS `Create` arm 不执行 webview 查找（D2），所以 ZST 的 key 冲突不影响此路径。但需确认没有其他代码路径用 `window_id` 做 lookup。
- **[Risk] `setWebController(newCtrl)` 创建的 controller 被泄漏** → `newCtrl` 被传给 ArkWeb 但不用于新窗口。ArkWeb 内部会管理此 controller 的生命周期。如果 ArkWeb 不释放它，可能造成内存泄漏。→ 缓解：观察 hilog 确认无泄漏迹象；若泄漏，可在 `setTimeout` 回调中显式 `newCtrl.destroy()`。
- **[Trade-off] `Create` 语义与桌面不同** → 桌面 `Create` = 用户提供 webview 实例；OHOS `Create` = 创建真窗口（不接受用户 webview）。这是平台架构限制，在设计文档和 spec 中显式标注。
- **[Trade-off] `NewWindowFeatures` 不传播** → OHOS `onWindowNew` 不提供 size/position。新窗口使用默认 800x600 尺寸。未来可通过 `onWindowNewExt`（API 12+）扩展。
