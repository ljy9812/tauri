## Context

wry 在 OHOS 上的 `InnerWebView`（`wry/src/ohos/mod.rs`）将 Cookie 操作委托给 `openharmony-ability::helper::webview::Webview`。当前：

- `cookies_for_url(url)` → 已实现：`Webview::cookies_with_url(url)` → ArkTS `getCookies(url)` → `WebCookieManager.fetchCookieSync(url)`，返回 `"; "` 分隔字符串，Rust 侧 `Cookie::parse` 拆分为 `Vec<Cookie>`。
- `cookies()` → `Ok(vec![])`（no-op）
- `set_cookie(_cookie)` → `Ok(())`（no-op）
- `delete_cookie(_cookie)` → `Ok(())`（no-op）

OHOS `WebCookieManager`（`@kit.ArkWeb`，静态方法，须 UI 线程）能力边界：
- `fetchCookieSync(url, incognito?)`（API 11+）：按 URL 取，返回 `"; "` 分隔串，无法取单条、无法枚举所有 URL
- `configCookieSync(url, value, incognito?)`（API 11+）：按 URL 设单条，`value` 须为 Set-Cookie 格式
- `clearAllCookiesSync(incognito?)` / `clearSessionCookieSync()`（API 11+）：仅全量清除
- `configCookieSync(url, value, incognito, includeHttpOnly)`（API 14+）、`saveCookieSync()`（API 15+）：不使用（避免版本守卫）

结论：OHOS **支持** 按 URL 设/取，**不支持** 枚举全部、**不支持** 删单条。本设计在能力边界内补齐 `set_cookie`，对 `cookies()`/`delete_cookie()` 显式降级。

## Goals / Non-Goals

**Goals:**
- `set_cookie()` 端到端可用（wry → ability NAPI → ArkTS `configCookieSync`）
- `cookies()` 提供 best-effort 行为（取当前 URL 的 Cookie），并显式标注平台限制
- `delete_cookie()` 安全降级（no-op + 告警），避免误用全量清除
- 遵守铁律：ArkTS 调用仅经 openharmony-ability；wry 改动限于 OHOS cfg 路径；API ≤ 12 无守卫

**Non-Goals:**
- 不实现「枚举所有 URL 的 Cookie」（OHOS 无 API）
- 不实现「删除单个 Cookie」（OHOS 无 API；不退化为全量清除）
- 不接入 API 14+/15+ 重载（`includeHttpOnly`、`saveCookieSync`）
- 不改动 `cookies_for_url` 与 `clear_all_browsing_data`
- 不暴露隐私模式（incognito）参数到 wry 公共 API

## Decisions

### D1: `set_cookie` 的 URL 推导
wry `Cookie` 含 `domain`、`path`、`name`、`value` 等。`configCookieSync` 需 `url` + Set-Cookie `value`。
- URL 推导：`https://{cookie.domain}`（domain 为空时回退当前 webview URL，再为空则报错跳过）
- value 格式化：`{name}={value}; Domain={domain}; Path={path||/}`，拼接 `Secure`/`SameSite` 等（若存在）
- 若 cookie 含 `Secure` 属性，URL 必须为 `https://`（已满足推导）

### D2: `cookies()` best-effort
- 取当前 webview URL（`self.webview.url()`），调 `cookies_with_url(current_url)`
- 无 URL / 取 URL 失败 → 返回 `Ok(vec![])`
- 与 Windows/macOS 的「全量」语义不同，design/spec 显式标注为平台限制降级

### D3: `delete_cookie` no-op + 告警
- 保持 `Ok(())`，增加 `log::warn!("[wry] delete_cookie is a no-op on OHOS: platform lacks single-cookie deletion")`
- 不调用 `clearAllCookies`（语义错误）。全量清除已由 `clear_all_browsing_data` 提供

### D4: NAPI 桥接命名（铁律：camelCase）
- Rust `Webview::set_cookie(url: String, value: String)` → ArkTS JsHelper `setCookie(url, value)`
- 复用现有 `getCookies` NAPI 调用模式（`get_named_property::<Function<FnArgs<(String,String)>, ()>>("setCookie")`）
- `cookies()` 复用 `getCookies(current_url)`，无需新 NAPI

### D5: 线程与调用时机
- `WebCookieManager` 静态方法须 UI 线程 → 复用 `get_main_thread_env()` 模式（与 `getCookies` 一致）
- `configCookieSync` 建议 Web 加载前调用；运行时设置依赖 OHOS 内部落盘（30s 周期），不强制 `saveCookieSync`

## Risks / Trade-offs

- **`cookies()` 语义不一致**：跨平台行为差异（OHOS 仅当前 URL）。→ 缓解：spec/manual 测试显式标注，文档说明
- **`delete_cookie` 静默 no-op**：开发者预期删除却不生效。→ 缓解：`log::warn!` + spec 标注；不静默
- **Set-Cookie 格式化偏差**：wry `Cookie` 字段映射到 Set-Cookie 可能不全（如 `HttpOnly`、`Max-Age`）。→ 缓解：映射核心字段（name/value/domain/path/secure/samesite），其余忽略并文档化
- **domain 为空时 URL 推导失败**：→ 缓解：回退当前 URL，仍失败则 `log::warn!` + `Ok(())` 跳过
- **不使用 API 14+ includeHttpOnly**：无法覆盖 http-only cookie。→ 接受（保持 API 12 兼容）
