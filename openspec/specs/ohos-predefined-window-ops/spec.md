# ohos-predefined-window-ops Specification

## Purpose
TBD - created by archiving change p1-predefined-multi-window. Update Purpose after archive.
## Requirements
### Requirement: 各操作的层级定义

OHOS 只有一个 tray icon（应用级入口），因此 predefined 菜单操作需明确区分 app 级与 window 级语义。对标 macOS 响应者链的设计：

| 操作 | 层级 | macOS 对标 | 说明 |
|------|------|-----------|------|
| Hide | **App 级** | `NSApplication.hide:` | 隐藏整个应用（所有窗口） |
| Minimize | **Window 级** | `NSWindow.performMiniaturize:` | 最小化当前焦点窗口 |
| CloseWindow | **Window 级** | `NSWindow.performClose:` | 关闭当前焦点窗口 |
| Maximize | **Window 级** | `NSWindow.performZoom:` | 最大化当前焦点窗口 |
| Fullscreen | **Window 级** | `NSWindow.toggleFullScreen:` | 全屏当前焦点窗口 |
| Recover | **Window 级** | 再次 `performZoom:` | 从最大化/全屏恢复 |

- App 级操作不依赖窗口焦点，直接对整个 Ability 生效
- Window 级操作需确定目标窗口，通过 `lastUserInteractedWindow` 追踪机制获取（基于 onTouch 事件，详见下文）

### Requirement: Window 级操作的目标窗口确定

Window 级操作（Minimize、CloseWindow、Maximize、Fullscreen、Recover）的目标窗口取决于调用来源：

| 调用来源 | `targetWindowId` | 目标窗口确定方式 |
|---------|------------------|----------------|
| **Window Menu Bar** | 有值（菜单所属窗口 ID） | 直接操作 `targetWindowId` 对应的窗口 |
| **Popup Menu** | 有值（右键所在窗口 ID） | 直接操作 `targetWindowId` 对应的窗口 |
| **Tray Menu** | `undefined` | 操作 `lastUserInteractedWindow`（用户最后触摸的窗口） |

- Window Menu Bar 和 Popup Menu 的菜单绑定在特定窗口上，用户正在操作该窗口，应直接作用于该窗口
- Tray Menu 是 app 级入口，不属于任何窗口，需通过 `lastUserInteractedWindow` 追踪用户之前交互的窗口

### Requirement: 所有 window 级操作统一通过 getTargetWindow() 解析目标窗口

`execute()` 方法中所有 window 级操作（minimize、close、maximize、fullscreen、recover）必须使用 `getTargetWindow(targetWindowId)` 解析目标窗口，禁止直接使用 `this.win`（主窗口）。

**原因**：`this.win` 始终是主窗口，在 Tray Menu 路径下（`targetWindowId` 为 `undefined`）会导致操作永远作用于主窗口，与 macOS Key Window 语义不一致。

`getTargetWindow()` 的解析优先级：
1. `targetWindowId` 有值 → Window Menu Bar / Popup Menu → 用该窗口
2. `lastUserInteractedWindow` 存在 → Tray Menu → 用最后交互的窗口
3. fallback → 主窗口

### Requirement: 用户交互窗口追踪——基于 onTouch，不使用 WINDOW_ACTIVE

OHOS 的 `WINDOW_ACTIVE` 事件是**级联广播**：父窗口获焦时，所有子窗口依次收到 `WINDOW_ACTIVE`。这导致 `WINDOW_ACTIVE` 无法可靠追踪"用户最后交互的窗口"。

**三平台对比**：

| | macOS | Windows | OHOS |
|---|---|---|---|
| 目标窗口概念 | Key Window | Foreground Window | lastUserInteractedWindow |
| 决定因素 | **用户点击** | **用户点击** / 应用设置 | **用户触摸（onTouch）** |
| Tray 菜单是否改变焦点 | ❌ 不变 | ❌ 不变 | ✅ 触发级联激活 |
| 子窗口是否级联 | ❌ 不级联 | ❌ 不级联 | ✅ 全部级联 |
| 可靠性 | ✅ 可靠 | ✅ 可靠 | ❌ WINDOW_ACTIVE 不可靠 |

**OHOS 解决方式**：使用 `onTouch` 事件追踪用户实际交互的窗口，与 macOS Key Window / Windows Foreground Window 语义一致。

1. 每个窗口的**页面根容器**（MainPage / FloatPage 的 root Stack）注册 `onTouch` 回调，利用 ArkUI onTouch 冒泡特性覆盖所有子组件（MenuBarComponent + DefaultXComponent + drag bar 等）
2. 用户触摸窗口内任意区域时，WindowManager 记录 `lastUserInteractedWindow`
3. Tray 菜单操作的目标窗口基于 `lastUserInteractedWindow` 确定

#### OHOS 平台差异
WINDOW_ACTIVE 的级联广播是 OHOS 系统行为（华为官方确认），无法通过应用层代码改变。macOS 和 Windows 的焦点机制不存在此问题。

#### 已知限制
系统 title bar（app 名 + 放大/缩小/关闭按钮）由 OHOS 系统渲染，不在 ArkUI 组件树内，无法通过 onTouch 监听。title bar 按钮的操作是系统级窗口行为，不走 predefined action 路径，不影响 lastUserInteractedWindow 的正确性。`inputMonitor.on('touch')` 可覆盖系统区域但需要 `ohos.permission.INPUT_MONITORING`（system_basic + AGC 审批），普通应用不可用。

### Requirement: hide predefined action 使用 hideAbility 隐藏应用（App 级）
当用户点击 Hide 菜单项时，系统调用 `hideAbility(want)` 隐藏整个应用到后台（所有窗口均不可见）。

#### Scenario: 用户点击 Hide
- WHEN 用户点击 Hide 菜单项
- THEN 调用 `context.hideAbility(want)` 隐藏应用（App 级，所有窗口）
- THEN 应用可通过托盘图标恢复

### Requirement: close 子窗口使用 destroyWindow（Window 级）
当用户点击 Close 且 `lastUserInteractedWindow` 为子窗口 (id > 0) 时，调用 `destroyWindow()` 关闭窗口。

#### Scenario: 关闭子窗口
- WHEN 用户点击 Close 且 targetWindowId > 0
- THEN 调用 `notifyWindowClose` + `removeWindow` + `destroyWindow`

### Requirement: close 主窗口使用 hideAbility（Window 级，主窗口特殊处理）
当用户点击 Close 且 `lastUserInteractedWindow` 为主窗口 (id = 0) 时，调用 `hideAbility()` 隐藏应用。主窗口不可 destroyWindow（WindowStage 会失效）。

#### Scenario: 关闭主窗口
- WHEN 用户点击 Close 且 targetWindowId = 0
- THEN 调用 `context.hideAbility(want)` 隐藏应用

### Requirement: 托盘图标点击恢复应用
当用户点击托盘图标时，调用 `showAbility(want)` 恢复应用。

#### Scenario: 点击托盘恢复
- WHEN 应用处于隐藏状态
- WHEN 用户点击托盘图标
- THEN 调用 `context.showAbility(want)` 恢复应用

### Requirement: minimize 最小化用户交互的窗口（Window 级）
Minimize 作用于 `lastUserInteractedWindow` 追踪到的窗口，调用 `win.minimize()`。

#### Scenario: 最小化主窗口
- WHEN 用户点击 Minimize 且最后交互窗口为主窗口
- THEN 主窗口最小化（OHOS 上等价于整个 Ability 进入后台，子窗口跟随）

#### Scenario: 最小化子窗口
- WHEN 用户点击 Minimize 且最后交互窗口为子窗口
- THEN 该子窗口隐藏（OHOS 子窗口最小化等价于隐藏，无 Dock 入口恢复）

#### OHOS 平台差异
- **主窗口 minimize**：OHOS 上主窗口 minimize 会触发整个 Ability 进入后台，所有子窗口跟随。这与 macOS 不同（macOS 只最小化当前窗口，子窗口保持可见）。此为 OHOS 系统行为，无法规避。
- **子窗口 minimize**：OHOS 子窗口 minimize 等价于 hide()，不会出现在 Dock 中，只能通过代码 `showWindow()` 恢复。

### Requirement: quit 行为不变
Quit 继续使用 `terminateSelf()` 退出应用。

### Requirement: 窗口操作时序——事件驱动，不使用 timeout

StatusBar 菜单点击会触发系统发送 `onNewWant`，导致系统在菜单操作执行后约 17ms 把应用拉回前台（GoForeground + Show）。如果 minimize/hide/closeWindow 立即执行，窗口会被操作后又恢复前台，产生"抖动"效果。

**解决方式**：使用 `WINDOW_ACTIVE` 事件驱动，不使用 timeout：

1. 菜单项点击时，将操作存入 `WindowManager.pendingAction`，**不立即执行**
2. WindowManager 在初始化时已为所有窗口注册了 `WINDOW_ACTIVE` 监听
3. 当系统完成 foreground 流程、触发 `WINDOW_ACTIVE` 时，执行 `pendingAction`

#### OHOS 平台差异
此行为是 OHOS StatusBar API 的副作用，macOS/Windows 无此问题。macOS 使用 responder chain，Windows 使用 WM_COMMAND，均不会触发额外的 foreground 事件。

### Requirement: WindowManager 包装方法统一管理 resetUserInteractionTracking

所有使窗口不可见的操作（minimize/hide/close/destroy）必须通过 WindowManager 包装方法执行，包装方法内部统一调用 `resetUserInteractionTracking()`。

**问题背景**：
- `lastUserInteractedWindow` 用于 Tray 菜单确定目标窗口（等价 macOS Key Window）
- 窗口被 minimize/hide/close 后需要 reset，否则 Tray 菜单会操作已不可见的窗口
- 原来 reset 散落在 `PredefinedActionExecutor.execute()` 中，Tauri API 路径（`WindowOpsExecutor`）未覆盖

**WindowManager 包装方法定义**：

| 包装方法 | 主窗口 (id=0) | 子窗口 (id>0) |
|----------|---------------|---------------|
| `minimizeWindow(id)` | win.minimize() + reset | win.minimize() + reset |
| `destroyWindow(id)` | win.destroyWindow() + removeWindow() + reset | win.destroyWindow() + removeWindow() + reset |
| `closeWindow(id)` | hideAbility() + reset | win.destroyWindow() + removeWindow() + reset |
| `hideAbility()` | context.hideAbility() + reset | N/A (Ability 级别) |

**调用方约束**：
- `PredefinedActionExecutor`（Menu/Tray 路径）：`minimizeWindow()` / `closeWindow()` 必须调用 WindowManager 包装方法
- `WindowOpsExecutor`（Tauri API 路径）：`minimize()` / `destroyWindow()` 必须调用 WindowManager 包装方法
- `PredefinedActionExecutor.hideAbility()`：薄包装，委托给 `WindowManager.hideAbility()`

**禁止行为**：
- 禁止在 WindowManager 包装方法之外直接调用 `win.minimize()` / `win.destroyWindow()` / `context.hideAbility()`
- 禁止在调用方手动调用 `resetUserInteractionTracking()`（由包装方法内部处理）

#### macOS 语义对齐
- macOS predefined menu 的 Minimize/Maximize/Close 通过 Responder Chain 操作 **Key Window**（当前焦点窗口）
- OHOS 的 `lastUserInteractedWindow`（onTouch-based）等价于 macOS Key Window 概念
- `closeWindow(id=0)` 调用 `hideAbility()` 对齐 macOS 主窗口 close 语义（隐藏而非销毁）

---

