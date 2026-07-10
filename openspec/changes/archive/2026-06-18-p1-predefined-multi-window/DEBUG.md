# p1-predefined-hide-close 问题追踪

## 问题描述

p1 实现后，在实际设备上发现两个 Bug：

### Bug 1：minimize/closeWindow 作用于错误窗口（有子窗口时）

**现象**：用户点击主窗口后打开 tray 菜单，点击 minimize/closeWindow，结果作用于子窗口而非主窗口。

**根因**：`getLastWindow()` 返回的是 **Z-order 最高（最顶层）的子窗口**，不是最后获焦的窗口。

当 tray 菜单项 click handler 执行时，所有窗口的 `isFocused()` 均返回 `false`（菜单抢走了焦点），`getFocusedWindow()` fallback 到 `getLastWindow()`，返回错误的子窗口。

**日志证据**（log20260617.log）：
```
10:11:02.248  OnIsFocused: window [620, test-close-req_1]  isFocused=0
10:11:02.248  OnIsFocused: window [621, test-destroyed_2]  isFocused=0
10:11:02.248  OnIsFocused: window [622, test-borderless]   isFocused=0
10:11:02.248  OnIsFocused: window [623, test-transparent]  isFocused=0
10:11:02.248  → Using getLastWindow as fallback           ← 命中错误 fallback
10:11:02.359  OnMinimize: subWindow or float window use hide  ← 子窗口被操作
10:11:02.361  UpdateFocus: focusId:618 (api0 主窗口)       ← 系统焦点实际在主窗口
```

**OHOS 官方说明**（arkts-helper 确认）：
> `window.getLastWindow()` 返回的是**当前应用内层级最高的子窗口**。如果应用当前没有子窗口，或者子窗口未调用 `showWindow()` 显示，则会返回应用主窗口。

### Bug 2：minimize/closeWindow/hide 有 ~1.5s 延迟（只有主窗口时）

**现象**：只有主窗口时，三个操作均有明显延迟后才执行。

**根因**：`waitForWindowActive()` 等待 `WINDOW_ACTIVE` 事件，但当只有主窗口时该事件**不触发**（系统跳过焦点切换），导致等待 1.5s 超时后才执行。

**日志证据**：
```
10:11:08.965  menu click → minimize
10:11:08.966  Using getLastWindow as fallback
10:11:10.467  waitForWindowActive: timeout, executing action  ← 1.5s 超时
10:11:10.468  OnMinimize Window [618, api0] minimize end       ← 终于执行
```

---

## 修复方案

### 核心思路：主动追踪最后获焦窗口，彻底替代 `getLastWindow()`

#### 1. WindowManager 增加焦点追踪

在 `WindowManager.addWindow()` 时，为每个窗口注册 `windowEvent` 监听，持续更新 `lastFocusedWindow`：

```typescript
// WindowManager 新增字段
private lastFocusedWindow: window.Window | null = null;
private lastFocusedWindowId: number = -1;

// addWindow() 中新增
win.on('windowEvent', (eventType: window.WindowEventType) => {
  if (eventType === window.WindowEventType.WINDOW_ACTIVE) {
    this.lastFocusedWindow = win;
    this.lastFocusedWindowId = windowId;
  }
});

// 新增公开方法
getFocusedWindow(): window.Window | null {
  return this.lastFocusedWindow;
}

getFocusedWindowId(): number {
  return this.lastFocusedWindowId;
}
```

#### 2. PredefinedActionExecutor 简化

**目标窗口确定逻辑（区分调用来源）：**

| 调用来源 | `targetWindowId` | 目标窗口确定方式 |
|---------|------------------|----------------|
| Window Menu Bar | 有值（菜单所属窗口 ID） | 直接操作 `targetWindowId` 对应的窗口 |
| Tray Menu | `undefined` | 操作 `lastFocusedWindow` |

- Window Menu Bar 的菜单绑定在特定窗口上，用户正在操作该窗口，应直接作用于该窗口
- Tray Menu 是 app 级入口，不属于任何窗口，需通过 `lastFocusedWindow` 追踪用户之前交互的窗口

**`minimizeWindow()` 和 `closeWindow()` 的策略链更新为：**

1. 如果 `targetWindowId` 有值 → 直接使用该窗口（Window Menu Bar 路径）
2. 否则使用 `WindowManager.getFocusedWindow()` — 菜单打开前的最后焦点窗口（**替代 `getLastWindow()`**）
3. 主窗口 `this.win` — 最终兜底

**三个操作全部直接执行，移除 `waitForWindowActive()`：**

- `minimizeWindow()` → `win.minimize()` 直接调用
- `closeWindow()` → 直接判断子窗口/主窗口，执行 `destroyWindow()` 或 `hideAbility()`
- `hideAbility()` → 直接 `context.hideAbility()`

`waitForWindowActive()` 方法整体删除。

---

## 第一轮修复后测试结果（第二轮问题发现）

按照上述方案实现后，测试发现以下新问题：

### Bug 3：主窗口 minimize 导致所有窗口都被最小化

**现象**：有子窗口时，点击主窗口 → 打开 tray 菜单 → minimize → 所有窗口（包括子窗口）都被最小化。

**日志证据**：
```
11:08:56.165  Using lastFocusedWindow: id=0       ← 正确选择主窗口
11:08:56.165  Minimize: id: 627                   ← 调用 minimize()
11:08:56.165  OnMinimize Window [627, api0] minimize end, ret=0  ← 成功
11:08:56.187  GoBackground: reason: 4             ← 整个 Ability 进入后台
11:08:56.187  Hide: Window hide [id:627]          ← 主窗口隐藏
```

**根因**：**OHOS 平台行为**——主窗口 minimize 会触发整个 Ability 进入后台，子窗口跟随主窗口的生命周期。这是系统层面的设计，不是代码 bug。

**结论**：OHOS 上主窗口 minimize 的行为与 macOS/Windows 不同。macOS 主窗口 minimize 只最小化该窗口，子窗口保持可见。OHOS 无法实现此行为。

### Bug 4：主窗口 closeWindow 隐藏所有窗口，但子窗口未被销毁

**现象**：点击主窗口 → close → 所有窗口隐藏；左键点击 tray icon 后主窗口和子窗口一起恢复。

**根因**：这是**设计行为**（spec 中已定义）。主窗口不能 destroyWindow（WindowStage 会失效），所以 closeWindow 对主窗口调用 hideAbility()。子窗口只是被隐藏，未被销毁，因此 showAbility() 后一起恢复。

### Bug 5：子窗口 minimize 后无法恢复（showAll / bringAllToFront 无效果）

**现象**：子窗口被 minimize 后，通过 tray 菜单点击 showAll 和 bringAllToFront 都无法恢复它。

**根因**：**Rust tray-icon crate 未将 `showAll` / `bringAllToFront` 转发到 ArkTS**。

`tray-icon/src/platform_impl/ohos/event.rs` 的 `execute_predefined_action` 函数中，match 语句只列出了部分 predefined action，`showAll` 和 `bringAllToFront` 未在其中，掉入 `_` 通配分支被丢弃：

```rust
// event.rs:107-120
fn execute_predefined_action(predefined_type: &str) {
    match predefined_type {
        "quit" => { ... }
        "minimize" | "hide" | "maximize" | "close" | "fullscreen" | "about"
        | "copy" | "cut" | "paste" | "selectAll" | "undo" | "redo"
        | "recover" => {
            openharmony_ability::statusbar::execute_predefined_action(predefined_type).ok();
        }
        _ => {  // ← showAll 和 bringAllToFront 掉到这里，被丢弃
            log::debug!("[TrayIcon] unsupported predefined action: {}", predefined_type);
        }
    }
}
```

**日志证据**：
```
11:28:18.937  OnMinimize: subWindow or float window use hide   ← 子窗口被 minimize（隐藏）
11:28:23.628  OnMinimize: subWindow or float window use hide
11:28:27.094  OnMinimize: subWindow or float window use hide
11:28:30.816  [TrayIcon] menu click → predefined: showAll      ← 用户点击 showAll
11:28:30.816  [TrayIcon] execute_predefined_action: showAll
11:28:30.816  [TrayIcon] unsupported predefined action: showAll ← ❌ Rust 拦截，未转发到 ArkTS
11:28:34.089  [TrayIcon] unsupported predefined action: bringAllToFront ← ❌ 同样被拦截
```

**Menu Bar 路径无此问题**：muda 的 OHOS 实现正确序列化了 `ShowAll → "showAll"` 和 `BringAllToFront → "bringAllToFront"`（`muda/src/platform_impl/ohos/mod.rs:328-331`），且 Menu Bar 点击由 ArkTS `MenuManager.handleItemClick()` 直接调用 `executor.execute()`，不经过 Rust 的 match 过滤。

**两条路径对比**：

| 路径 | 是否支持 showAll/bringAllToFront | 原因 |
|------|------|------|
| Menu Bar（muda → ArkTS） | ✅ | ArkTS 直接调用 executor.execute()，case 'showAll' 有处理 |
| Tray（tray-icon → Rust → ArkTS） | ❌ | Rust match 未列出，被 `_` 丢弃 |

**修复**：在 `event.rs` 第 112 行 match arm 中加入 `"showAll" | "bringAllToFront"`。

### Bug 6：竞态问题——minimize/hide/closeWindow 操作后窗口抖动并恢复前台（最严重）

**现象**：点击 minimize/hide/closeWindow 后，窗口先被操作，然后抖动一下又恢复到前台。

**日志证据**（以 minimize 为例）：
```
11:08:56.165  Minimize: id: 627                   ← 我们调用 minimize
11:08:56.165  OnMinimize minimize end, ret=0      ← 成功
11:08:56.182  targeState:5, isNewWant:1           ← ⚠️ 系统发送新 Want（~17ms 后）
11:08:56.183  onNewWant                           ← 应用收到新意图
11:08:56.183  GoForeground: reason: 4             ← ❌ 系统把应用拉回前台！
11:08:56.183  Show: Window show [api0, id: 627]   ← 窗口重新显示
11:08:56.187  GoBackground: reason: 4             ← 最终进入后台（经过完整生命周期循环）
```

**根因**：**StatusBar 菜单点击触发系统发送 `onNewWant`**，系统因此把应用拉到前台（GoForeground + Show）。这发生在我们的 minimize/hide 操作之后约 17ms，导致窗口先被最小化/隐藏，然后又被系统拉回前台，产生"抖动"效果。

**时序分析**：
```
T+0ms     用户点击菜单项
T+0ms     menu click handler 执行 → minimize/hide 立即执行
T+17ms    系统发送 onNewWant（StatusBar 菜单关闭的副作用）
T+18ms    GoForeground → Show → 窗口恢复前台 ← 竞态！
T+22ms    GoBackground → Hide → 最终生效
```

### 第二轮修复方案（最终版）

**核心原则**：窗口生命周期与 Ability 生命周期分离，禁止在系统回调并行执行互斥窗口前后台操作（参考 `window_lifecycle.md`）。

**方案：事件驱动 + 安全网定时器**

核心思路：不在菜单同步回调执行窗口操作，而是在 `onNewWant` 区分托盘来源后，等前台流程完成再执行。

#### 执行流程

```
Tray Menu 路径（StatusBar 菜单 → 系统 UI）：

  ① 菜单点击 → 设置 pendingAction（不执行任何窗口 API）
               → 启动安全网定时器（2s）
  
  ② 系统关闭菜单 → 触发 onNewWant（~17ms 后）
  
  ③ onNewWant 检测到 pendingAction：
     → 取消安全网定时器
     → 注册一次性 windowEvent 监听（RESUMED 或 WINDOW_ACTIVE）
  
  ④ 事件触发（系统前台流程完成）→ 执行窗口操作 → 清理状态

Window Menu 路径（窗口内菜单栏 → 不触发 onNewWant）：

  菜单点击 → 直接执行窗口操作（无竞态风险）
```

#### 代码设计

```typescript
// 全局状态
let pendingAction: (() => void) | null = null;
let cleanupTimer: number | null = null;

// ① 菜单点击时（Tray 路径）
function onTrayMenuClick(actionType: string): void {
  // 设置待执行操作
  pendingAction = () => executeWindowAction(actionType);
  
  // 启动安全网定时器（2s）
  cleanupTimer = setTimeout(() => {
    if (pendingAction) {
      hilog.warn(DOMAIN, 'Menu', 'pendingAction timeout, cleaning up');
      pendingAction = null;
    }
  }, 2000);
}

// ③ onNewWant 中检测
function onNewWant(want: Want): void {
  if (pendingAction) {
    // 取消安全网定时器
    if (cleanupTimer !== null) {
      clearTimeout(cleanupTimer);
      cleanupTimer = null;
    }
    
    // 注册一次性事件监听
    const win = getCurrentWindow();
    const handler = (eventType: window.WindowEventType) => {
      if (eventType === window.WindowEventType.RESUMED ||
          eventType === window.WindowEventType.WINDOW_ACTIVE) {
        win.off('windowEvent', handler);
        
        const action = pendingAction;
        pendingAction = null;
        action?.();  // ④ 系统前台流程完成，安全执行
      }
    };
    win.on('windowEvent', handler);
  }
}
```

#### 时序分析

**正常流程（99% 情况）**：
```
T+0ms     设置 pendingAction + 启动清理定时器（2s）
T+17ms    onNewWant 触发 → 取消定时器 → 注册事件监听
T+18ms    RESUMED 触发 → 执行操作 → 清除状态 ✅
          （定时器永远不会触发）
```

**异常流程（1% 情况）**：
```
T+0ms     设置 pendingAction + 启动清理定时器（2s）
T+17ms    onNewWant 没触发（极端情况）
...
T+2000ms  定时器触发 → 检测到 pendingAction 仍在 → 记录日志 → 清除状态
          （防止状态永久污染，用户需重新点击菜单）
```

#### 方案优势

| 特性 | 说明 |
|------|------|
| **事件驱动** | 窗口操作在 RESUMED/WINDOW_ACTIVE 后执行，不与系统前台化竞争 |
| **无执行定时器** | 定时器仅用于清理状态，不用于执行窗口操作 |
| **路径分离** | Tray 路径延迟执行，Window Menu 路径直接执行 |
| **安全网** | 极端情况下清理状态，防止永久卡死 |
| **符合官方实践** | 华为官方推荐窗口事件监听方案（精度最高） |

#### 与第一版方案对比

| 版本 | 方案 | 问题 |
|------|------|------|
| 第一版 | `waitForWindowActive` + 1.5s timeout 执行 | 只有主窗口时 WINDOW_ACTIVE 不触发，timeout 后执行有延迟 |
| 第二版（旧） | `pendingAction` + 永久 WINDOW_ACTIVE 监听，无 timeout | WINDOW_ACTIVE 不触发时 pendingAction 永久卡住 |
| **第二版（最终）** | `pendingAction` + onNewWant 检测 + RESUMED 执行 + 2s 清理定时器 | ✅ 无已知问题 |

**关键改进**：
- 使用 RESUMED 作为主要触发事件（标记前台流程完成），WINDOW_ACTIVE 作为备选
- 通过 onNewWant 检测触发时机（而非依赖永久监听）
- 清理定时器仅用于异常兜底，正常流程中会被取消

---

### Bug 7：WINDOW_ACTIVE 级联广播导致 lastFocusedWindow 追踪错误（根本性缺陷）

**现象**：有子窗口时，用户点击主窗口 → 打开 Tray 菜单 → 点击 Minimize/CloseWindow → 操作的是子窗口而非主窗口。

**日志证据**：
```
14:12:38.022  Last focused window updated: id=0     ← 用户点击主窗口 ✅
14:12:38.045  Last focused window updated: id=3     ← 23ms 后子窗口 3 获焦（用户未点击！）
14:12:38.047  Last focused window updated: id=4     ← 2ms 后子窗口 4 获焦（用户未点击！）

14:12:43.779  setPendingAction()                    ← 此时 lastFocusedWindow 已是 id=4
14:12:43.893  Consuming pending action              ← 读取 lastFocusedWindow = id=4
14:12:43.893  OnMinimize: subWindow use hide        ← ❌ 操作了子窗口
```

**根因**：**OHOS 的 WINDOW_ACTIVE 是级联广播事件，不适合作为追踪用户交互窗口的唯一依据。**

华为官方确认：
- 当主窗口获得焦点时，系统将其**整个窗口树**提升到前台，并**依次激活父窗口及所有子窗口**
- WINDOW_ACTIVE 不仅在用户点击时触发，还在以下场景触发：
  - 父窗口被激活 → 子窗口级联激活
  - 窗口被程序化调用 `showWindow()` 或 `raiseToTop()`
  - 应用从后台切换回前台（GoForeground）

**与 macOS / Windows 的对比**：

| | macOS | Windows | OHOS |
|---|---|---|---|
| 目标窗口概念 | Key Window | Foreground Window | lastFocusedWindow |
| 决定因素 | **用户点击** | **用户点击** / 应用设置 | WINDOW_ACTIVE 事件 |
| Tray 菜单是否改变焦点 | ❌ 不变 | ❌ 不变 | ✅ 触发级联激活 |
| 子窗口是否级联 | ❌ 不级联 | ❌ 不级联 | ✅ 全部级联 |
| 可靠性 | ✅ 可靠 | ✅ 可靠 | ❌ 不可靠 |

**macOS**：Key Window = "current focus of user input"，仅由用户点击决定，Tray 菜单操作通过 responder chain 路由到 Key Window，打开 Tray 菜单不改变 Key Window。（来源：[Apple Event Architecture](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/EventOverview/EventArchitecture/EventArchitecture.html)、[Cocoa AppKit Responder Chain](https://christiantietze.de/posts/2023/08/cocoa-appkit-responder-chain/)）

**Windows**：Foreground Window 由用户点击或应用显式 `SetForegroundWindow()` 决定，Tray 菜单本身不改变应用窗口的前台状态。（来源：[Microsoft TrackPopupMenu](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-trackpopupmenu)）

**冻结方案（frozenFocusTarget）的局限**：
- 只能防止 consumePendingAction 期间被覆盖
- 如果 setPendingAction 时 lastFocusedWindow 已经被级联事件覆盖（如上述日志），冻结的也是错误值
- 本质是 workaround，不是根本解决方案

### Bug 7 修复方案：onTouch 追踪用户交互窗口

**核心思路**：用 `onTouch` 事件替代 `WINDOW_ACTIVE` 追踪用户实际交互的窗口，与 macOS Key Window / Windows Foreground Window 行为一致。

**方案**：

1. 每个窗口的顶层组件上注册 `onTouch` 回调
2. 用户触摸窗口时，记录 `lastUserInteractedWindowId`
3. Tray 菜单操作的目标窗口基于 `lastUserInteractedWindowId` 确定

```typescript
// WindowManager 新增
private lastUserInteractedWindowId: number = -1;
private lastUserInteractedWindow: window.Window | null = null;

setUserInteractedWindow(win: window.Window, windowId: number): void {
  this.lastUserInteractedWindow = win;
  this.lastUserInteractedWindowId = windowId;
  hilog.info(DOMAIN, 'WindowManager',
    'User interacted with window: id=%{public}d', windowId);
}

getUserInteractedWindow(): window.Window | null {
  return this.lastUserInteractedWindow;
}

getUserInteractedWindowId(): number {
  return this.lastUserInteractedWindowId;
}
```

```typescript
// 在每个窗口的 WebView 容器组件中（DefaultXComponent.ets 或类似）
.onTouch(() => {
  const wm = WindowManager.getInstance();
  wm.setUserInteractedWindow(this.window, this.windowId);
})
```

**与 WINDOW_ACTIVE 追踪的对比**：

| | WINDOW_ACTIVE 追踪 | onTouch 追踪 |
|---|---|---|
| 用户点击主窗口 | ✅ main | ✅ main |
| 子窗口级联激活 | ❌ 被覆盖为 sub | ✅ 不变（无 touch） |
| GoForeground | ❌ 被覆盖 | ✅ 不变（无 touch） |
| 从后台恢复后无 touch | ❌ 不确定 | ✅ 保持上次值 |
| 从后台恢复后点击主窗口 | ❌ 可能被级联覆盖 | ✅ main |

**边缘情况**：
- 从后台恢复后，用户未触摸任何窗口就直接操作 Tray 菜单 → 沿用上次触摸的窗口（合理）
- 从后台恢复后，用户触摸了主窗口 → lastUserInteractedWindow 更新为主窗口（正确）

---

## 涉及文件

| 文件 | 改动 |
|------|------|
| `WindowManager.ets` | 新增 `lastFocusedWindow` 追踪 + 监听注册 + pendingAction 管理 + frozenFocusTarget |
| `menu.ets` | 简化 `getFocusedWindow()`，移除 `waitForWindowActive()` |
| `ArkHelper.ets` | `executePredefinedAction` 窗口操作延迟执行 |
| `NativeAbility.ets` | `onNewWant` 取消清理定时器 |
| `tray-icon/event.rs` | match arm 加入 `showAll` / `bringAllToFront` |

### 待实现（Bug 7）

| 文件 | 改动 |
|------|------|
| `WindowManager.ets` | 新增 `lastUserInteractedWindow` 追踪，替代 WINDOW_ACTIVE |
| `DefaultXComponent.ets` | 添加 `onTouch` 回调记录用户交互窗口 |

## 验证标准

- [ ] 有子窗口时：点击主窗口 → minimize → 主窗口最小化（子窗口不受影响）
- [ ] 有子窗口时：点击主窗口 → closeWindow → 主窗口隐藏（hideAbility）
- [ ] 只有主窗口时：minimize → 立即执行，无抖动
- [ ] 只有主窗口时：hide → 立即执行，无抖动
- [ ] Tray 菜单 ShowAll → 恢复被隐藏的窗口
- [ ] Tray 菜单 BringAllToFront → 恢复被隐藏的窗口
- [ ] 有子窗口时：点击主窗口 → 打开 Tray 菜单 → Minimize → 操作主窗口（Bug 7）
- [ ] 从后台恢复后：不触摸窗口 → Tray Minimize → 操作上次触摸的窗口（Bug 7）
- [ ] 从后台恢复后：触摸主窗口 → Tray Minimize → 操作主窗口（Bug 7）
- [ ] 子窗口获焦时：点击子窗口 → minimize → 该子窗口隐藏
