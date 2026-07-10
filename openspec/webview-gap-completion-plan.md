# WebView 功能补全适配计划

**创建时间**：2026-06-26
**功能描述**：补全龙剑吟负责的 WebView 相关功能缺口（已排除低优先级 10 项与已实现 R70 User Agent / R76 可见性 / R94 背景颜色）。覆盖 Cookie 管理、DevTools、渲染收尾、拖拽、打印、新窗口 Create、桌面系功能。
**目标设备形态**：含 OHOS 桌面/大屏（mobile + desktop）
**判断依据**：涉及 3 个代码层（wry / openharmony-ability Rust NAPI / ArkTS），预估 ~20 文件，既有底层 NAPI 缺口又有上层 wry 接通 → 拆分
**目标级别**：完整实现（ feasible 项）；平台不支持项显式标注降级

## 缺口来源（核查后）

| 簇 | 行 | 功能 | 现状 | 关键缺口 |
|----|----|------|------|---------|
| Cookie | R69/R81/R96/R97 | Cookie 管理/增删查 | ⚠️/❌ | cookies/set_cookie/delete_cookie 为 no-op |
| DevTools | R71/R73 | 开发者工具/状态 | ⚠️ | open/close_devtools 空体；is_devtools_open 恒 false |
| 渲染 | R74/R78 | 透明背景/边界 | ⚠️ | R74 核实 ArkHelper 是否闭环；R78 非子 set_bounds 仅缓存 |
| 拖拽 | R72 | 文件拖拽 | ⚠️ | drag_drop_handler 未接通（feature flag + drag.rs 已存在）|
| 打印 | R83 | 打印 | ❌ | print 空体（create_pdf 已可用，可复用）|
| 新窗口 | R87 | 新窗口 Create | ⚠️ | NewWindowResponse::Create 降级为 Allow |
| 桌面系 | R75/R77/R82/R85/R86/R90/R91 | HTTPS协议/聚焦/剪贴板/数据存储标识/数据目录/点击穿透/热键缩放 | ❌ | 仅桌面；含桌面形态后纳入评估实现 |

## Phase 列表

| Phase | 名称 | openspec change | 状态 | 涉及层 | 预估文件 | 验证方式 |
|-------|------|----------------|------|--------|---------|---------|
| 1 | Cookie 管理补全 | `p1-webview-cookie` | ✓ 已归档 | wry+ability+ArkTS | 5 | 设备端 cookie 读/写/删用例 |
| 2 | DevTools 状态 + 聚焦父窗口 | `p2-webview-devtools-focus` | ✓ 已归档 | wry+ArkTS | 3 | is_devtools_open 返回真实值 + focus 生效 |
| 3 | 渲染收尾（透明背景核实 + 边界） | `p3_webview-render-finish` | ✓ 已归档 | wry+ArkTS | 3 | 透明窗口可见 + 主 webview set_bounds 生效 |
| 4 | 文件拖拽 | `p4_webview-drag-drop` | ✗ 平台限制 | wry+ability+ArkTS | 4 | NWeb 消费拖拽事件，HTML5 DnD 原生可用 |
| 5 | 打印 | `p5_webview-print` | ○ 待开始 | wry+ArkTS | 3 | print 生成 PDF / 触发系统打印 |
| 6 | 新窗口 Create 变体 | `p6-webview-new-window-create` | ✓ 设计完成 | wry+ability+ArkTS+tauri | 6 | window.open 真正创建子 webview 窗口 |
| 7 | 桌面系功能适配 | `p7_webview-desktop-features` | ✓ 设计完成 | wry+ability+ArkTS | 6 | 逐项适配或标注不适用（含桌面形态） |

## Phase 详细说明

### Phase 1: Cookie 管理补全
- **目标**：实现 `cookies()`（获取全部）、`set_cookie()`、`delete_cookie()`，替换 wry 层 3 个 no-op，打通 wry → ability NAPI → ArkTS Web Cookie API
- **文件列表**：
  - `wry/src/ohos/mod.rs`（cookies/set_cookie/delete_cookie 替换 stub）
  - `openharmony-ability/crates/ability/src/helper/webview.rs`（新增 3 个 NAPI 方法）
  - `openharmony-ability/native_ability/src/main/ets/webview/Utils.ets`（JsHelper + ProxyJsHelper 接口）
  - `openharmony-ability/native_ability/src/main/ets/webview/DefaultWebview.ets`（buildJsHelper 实现）
  - `openharmony-ability/native_ability/src/main/ets/webview/type.ets`（如需扩展 WebviewInitData）
- **依赖**：无（Step 3 需用 arkts-helper 查证 OHOS Web Cookie API，是最大未知项）
- **风险**：OHOS Web Cookie API 能力与版本需确认；若官方无「获取全部 cookie」API，需降级为遍历已知 URL

### Phase 2: DevTools 状态 + 聚焦父窗口
- **目标**：`is_devtools_open()` 返回 `setWebDebuggingAccess` 实际开关；`open/close_devtools` 映射为切换调试访问（移动端无独立 devtools 窗口，标注平台限制）；`focus_parent()` 复用已有 `ability::helper::webview::focus()`（requestFocus）
- **文件列表**：
  - `wry/src/ohos/mod.rs`（is_devtools_open/open_devtools/close_devtools/focus_parent）
  - `openharmony-ability/crates/ability/src/helper/webview.rs`（新增 devtools 状态查询，focus 已存在）
  - `openharmony-ability/native_ability/src/main/ets/webview/DefaultWebview.ets`（维护调试访问状态）
- **依赖**：无

### Phase 3: 渲染收尾（透明背景核实 + 边界）
- **目标**：核实 archive `p1-webview-transparent` 在 `ArkHelper.ets` 的 transparent 处理是否闭环（R74 若已闭环则关闭）；R78 让非子 webview 的 `set_bounds` 真正调用 ArkTS `setBounds` 而非仅缓存
- **文件列表**：
  - `wry/src/ohos/mod.rs`（set_bounds 非子分支）
  - `openharmony-ability/native_ability/src/main/ets/webview/DefaultWebview.ets`（主 webview setBounds）
  - `openharmony-ability/native_ability/src/main/ets/ArkHelper.ets`（核实 transparent）
- **依赖**：无

### Phase 4: 文件拖拽
- **目标**：激活 `drag_and_drop` feature flag，wry 层接 `drag_drop_handler`，ability `drag.rs` + ArkTS `onDragAndDrop` 打通文件拖入事件
- **文件列表**：
  - `wry/src/ohos/mod.rs`（drag_drop_handler 接通）
  - `openharmony-ability/crates/ability/src/webview/drag.rs`（激活）
  - `openharmony-ability/crates/ability/src/helper/webview.rs`（桥接）
  - `openharmony-ability/native_ability/src/main/ets/webview/DefaultWebview.ets`（onDragAndDrop）
- **依赖**：无

### Phase 5: 打印
- **目标**：`print()` 从 no-op 改为实际实现，复用已有 `create_pdf`（web→PDF）并接 OHOS 打印服务，或直接生成 PDF
- **文件列表**：
  - `wry/src/ohos/mod.rs`（print）
  - `openharmony-ability/crates/ability/src/helper/webview.rs`（print NAPI）
  - `openharmony-ability/native_ability/src/main/ets/webview/DefaultWebview.ets`/`Utils.ets`（print 实现）
- **依赖**：无

### Phase 6: 新窗口 Create 变体
- **目标**：实现 `NewWindowResponse::Create` 真正创建子 webview 窗口（当前 `mod.rs:225` 降级为 Allow），走 openharmony-ability 窗口创建管线
- **文件列表**：
  - `wry/src/ohos/mod.rs`（Create 变体处理）
  - `openharmony-ability/crates/ability/src/helper/webview.rs`
  - `openharmony-ability/crates/ability/src/window/mod.rs`（窗口创建）
  - `openharmony-ability/native_ability/src/main/ets/webview/NewWindowDialog.ets`（子窗 webview）
  - `openharmony-ability/native_ability/src/main/ets/WindowManager.ets`
- **依赖**：多窗口创建基础设施；复杂度最高，放最后
- **降级**：若 OS 级窗口创建暂不可用，保持 Allow 降级并显式标注

### Phase 7: 桌面系功能适配
- **目标**：含桌面形态后，逐项适配 R75（HTTPS 自定义协议）/R77（聚焦，可能与 Phase 2 合并）/R82（剪贴板，用 `@ohos.pasteboard`）/R85（数据存储标识）/R86（数据目录）/R90（点击穿透）/R91（热键缩放）；无法映射项标注不适用
- **文件列表**：
  - `wry/src/ohos/mod.rs`（各 feature 接入）
  - `openharmony-ability/crates/ability/src/helper/webview.rs`
  - `openharmony-ability/native_ability/src/main/ets/webview/DefaultWebview.ets`
  - 其他按 feature 增补
- **依赖**：Phase 1-6 完成；逐项评估后细化

## 状态说明
- `○ 待开始` — 未开始设计
- `● 进行中` — 正在设计或实现
- `✓ 设计完成` — 设计文档已生成并通过审计
- `✓ 已归档` — 已完成实现、测试并归档

## 备注
- arkts-helper MCP 工具在当前会话不可用，Step 3 探索将以 webfetch 查阅华为官方文档 + 本地 grep 代替
- 每个 Phase 完成设计后，使用 tauri-ohos-apply Skill 进入实现
