## Context

OHOS `hide`/`close`/`minimize` 三者 fallthrough 到同一个 `minimizeWithRestoreGuard(win)`，导致行为不正确。

## Decisions

### D1: hide 使用 hideAbility()
- `UIAbilityContext.hideAbility(want)` API 9+，无特殊权限
- 对标 macOS `hide:` selector

### D2: close 主窗口使用 hideAbility()
- `destroyWindow()` 后 WindowStage 无效，无法重建
- hideAbility 是 OHOS 上的最佳近似

### D3: close 子窗口使用 destroyWindow()
- 子窗口可以正常销毁，已有完整路径

### D4: 托盘点击自动 showAbility()
- 华为官方推荐模式
- `UIAbilityContext.showAbility(want)` API 9+

### D5: Want 使用 context.abilityInfo 动态构造
- 不需要硬编码 bundleName/abilityName

### D6: WindowManager 包装方法统一管理 resetUserInteractionTracking

**问题背景**：
- `lastUserInteractedWindow`（基于 onTouch）用于 Tray 菜单确定目标窗口（等价 macOS Key Window）
- 窗口被 minimize/hide/close 后需要 reset，否则 Tray 菜单会操作已不可见的窗口
- 原来 reset 散落在 `PredefinedActionExecutor.execute()` 中，Tauri API 路径（`WindowOpsExecutor`）未覆盖

**方案**：在 WindowManager 添加包装方法，所有使窗口不可见的操作统一通过包装方法，内部调用 reset：

| 包装方法 | 主窗口 (id=0) | 子窗口 (id>0) |
|----------|---------------|---------------|
| `minimizeWindow(id)` | win.minimize() + reset | win.minimize() + reset |
| `destroyWindow(id)` | win.destroyWindow() + removeWindow() + reset | win.destroyWindow() + removeWindow() + reset |
| `closeWindow(id)` | **hideAbility()** + reset | win.destroyWindow() + removeWindow() + reset |
| `hideAbility()` | context.hideAbility() + reset | N/A (Ability 级别) |

**调用方统一使用包装方法**：
- `PredefinedActionExecutor`（Menu/Tray 路径）：`minimizeWindow()` / `closeWindow()` → WindowManager wrappers
- `WindowOpsExecutor`（Tauri API 路径）：`minimize()` / `destroyWindow()` → WindowManager wrappers
- `PredefinedActionExecutor.hideAbility()`：薄包装，委托给 `WindowManager.hideAbility()`

**macOS 语义对齐**：
- macOS predefined menu 的 Minimize/Maximize/Close 通过 Responder Chain 操作 **Key Window**（当前焦点窗口）
- OHOS 的 `lastUserInteractedWindow`（onTouch-based）等价于 macOS Key Window 概念
- `closeWindow(id=0)` 调用 `hideAbility()` 对齐 macOS 主窗口 close 语义（隐藏而非销毁）

### D7: 所有 window 级操作统一使用 getTargetWindow() 确定目标窗口

**问题背景**：
`execute()` 方法中有两种窗口解析方式，导致行为不一致：

```
// 方式 A（有问题）—— maximize/fullscreen/recover 使用
let win = this.win;  // 默认主窗口
if (targetWindowId !== undefined && targetWindowId !== 0) {
  win = wm.getWindow(targetWindowId) ?? this.win;
}

// 方式 B（正确）—— minimize/close 使用
const { win, windowId } = this.getTargetWindow(targetWindowId);
```

**Tray 路径下 `targetWindowId` 是 `undefined`**：
- 方式 A：条件失败 → `win = this.win`（主窗口）→ 永远操作主窗口 ❌
- 方式 B：走 fallback chain → `lastUserInteractedWindow` → 正确目标窗口 ✓

**三种菜单的 targetWindowId 差异**：

| 菜单类型 | targetWindowId | 目标窗口 |
|----------|---------------|----------|
| Window Menu Bar | 有值（菜单所属窗口） | 该窗口 |
| Popup Menu | 有值（右键所在窗口） | 该窗口 |
| Tray Menu | undefined | lastUserInteractedWindow |

**修复**：`execute()` 顶部统一使用 `getTargetWindow(targetWindowId)` 解析目标窗口，所有 window 级操作（minimize/close/maximize/fullscreen/recover）共享解析结果。

**macOS 对齐**：macOS 所有 window 级 predefined selector（performMiniaturize:/performZoom:/performClose:/toggleFullScreen:）通过 Responder Chain 统一操作 Key Window。

### D8: onTouch 从 DefaultXComponent 迁移到页面根容器

**问题背景**：
`onTouch` 当前只注册在 `DefaultXComponent`（webview 容器）上。主窗口中 MenuBarComponent（label bar + menubar）和系统 title bar 区域的触摸不会触发 `setUserInteractedWindow()`，导致 Tray 菜单无法正确识别目标窗口。

**根因分析**：
```
MainPage / FloatPage
  Stack (root)
    Column
      MenuBarComponent       ← 无 onTouch，点击不记录 lastInteract ❌
      DefaultXComponent      ← onTouch 在这里，只有 webview 区域记录 ✅
```

**ArkUI onTouch 冒泡特性**（官方文档确认）：
- 父子组件 onTouch 事件**同时触发**，不竞争
- onTouch 与 Gesture 不竞争（不影响 drag bar 的 PanGesture 等）
- 鼠标左键点击被系统转换为 TouchEvent

**方案**：将 `onTouch` 从 `DefaultXComponent` 迁移到 MainPage / FloatPage 的根 `Stack`，利用冒泡覆盖所有子组件。

**覆盖范围**：

| 区域 | 修复前 | 修复后 |
|------|--------|--------|
| MenuBarComponent (label + menus) | ❌ 不记录 | ✅ 冒泡到 root |
| DefaultXComponent (webview) | ✅ 记录 | ✅ 冒泡到 root |
| FloatPage drag bar / close button | ❌ 不记录 | ✅ 冒泡到 root |
| FloatPage resize handles | ❌ 不记录 | ✅ 冒泡到 root |
| 系统 title bar (app名 + ─ □ ✕) | ❌ 不记录 | ❌ 不可覆盖（系统渲染，不在 ArkUI 组件树内） |

**系统 title bar 不可覆盖的合理性**：
- title bar 按钮（放大/缩小/关闭）是系统级窗口操作，不走我们的 predefined action 路径
- 用户点击 title bar 后使用 Tray 菜单，fallback 到主窗口是合理的（只有主窗口有系统 title bar）

**改动清单**：
1. `MainPage.ets`：根 `Stack` 加 `.onTouch()` → `setUserInteractedWindow(0)`
2. `FloatPage.ets`：根 `Stack` 加 `.onTouch()` → `setUserInteractedWindow(this.windowId)`
3. `DefaultXComponent.ets`：移除 `.onTouch()`（避免重复调用）
4. `WindowManager.ets`：更新注释

**已排除的替代方案**：
- `inputMonitor.on('touch')`：全局触摸监听，可覆盖系统 title bar，但需要 `ohos.permission.INPUT_MONITORING`（system_basic + AGC 审批），普通应用不可用
- 隐藏系统 title bar + 自定义：太侵入，改变窗口外观

## Risks

| Risk | Mitigation |
|------|------------|
| showAbility 失败 | try/catch + hilog warning |
| hideAbility 失败 | try/catch + hilog warning |
| WINDOW_ACTIVE 事件不可靠（级联广播） | 改用 onTouch 追踪 lastUserInteractedWindow |
| onTouch 在根容器冒泡影响性能 | 仅处理 TouchType.Down，Move/Up/Cancel 只过一个 if 判断 |
| onTouch 与子组件 Gesture 冲突 | ArkUI 文档确认 onTouch 与 Gesture 不竞争 |
