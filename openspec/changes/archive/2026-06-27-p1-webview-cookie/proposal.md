## Why

wry OHOS WebView 的 Cookie 管理存在 3 个 no-op：`cookies()`（获取全部）、`set_cookie()`、`delete_cookie()`；仅 `cookies_for_url()` 经 `WebCookieManager.fetchCookieSync` 可用。开发者无法在 OHOS 上设置或管理 Cookie，与 Windows/macOS 行为不一致。

经查证 OHOS `@kit.ArkWeb` 的 `WebCookieManager`：提供按 URL 取/设 Cookie 与全量清除能力，但**不提供「枚举所有 URL 的 Cookie」与「删除单个 Cookie」**接口。因此本 Phase 在 OHOS API 能力边界内补齐可实现项，对平台不支持项显式降级并标注。

## What Changes

- **实现 `set_cookie()`**：经 `WebCookieManager.configCookieSync(url, value)` 设置单个 Cookie。从 wry `Cookie` 的 `domain` 推导 URL（`https://<domain>`），将 `Cookie` 格式化为 Set-Cookie 字符串（`name=value; Domain=...; Path=...`）作为 `value`。使用 API 11+ 的 3 参重载（满足最低 API 12）。
- **`cookies()` 降级实现**：OHOS 无「枚举全部 Cookie」API。改为对当前 webview URL 调 `fetchCookieSync(current_url)` 返回该 URL 下 Cookie（best-effort）；无 URL 时返回空 `Vec`。在文档与备注显式标注此为平台限制。
- **`delete_cookie()` 降级为 no-op + 告警**：OHOS 仅提供 `clearAllCookies`（全量）与 `clearSessionCookie`，无单条删除。单条删除若改用全量清除语义错误，故保持 no-op 并 `log::warn!`，备注标注平台限制。
- **新增 openharmony-ability NAPI 桥接**：`helper/webview.rs` 新增 `set_cookie(url, value)`；ArkTS `Utils.ets` JsHelper 新增 `setCookie`（复用现有 `getCookies` 模式）。`cookies()`/`delete_cookie()` 不需新 NAPI（前者复用 `getCookies(current_url)`，后者纯 Rust no-op）。
- 不改动 `cookies_for_url()`（已可用）。

## Capabilities

### New Capabilities
- `webview-cookie-mgmt`: OHOS WebView Cookie 写入与按 URL 读取，含平台限制下的降级行为（全量枚举/单条删除不支持）

### Modified Capabilities
（无现有 spec 需修改——`cookies_for_url` 已实现，本 Phase 仅补齐 set/全量/单删）

## Impact

- **wry**（Rust）：`src/ohos/mod.rs` 的 `cookies()`/`set_cookie()`/`delete_cookie()` 替换 no-op；`set_cookie` 内做 Cookie→Set-Cookie 格式化与 URL 推导
- **openharmony-ability**（Rust）：`crates/ability/src/helper/webview.rs` 新增 `set_cookie(url: String, value: String)` NAPI 方法
- **openharmony-ability**（ArkTS）：`native_ability/src/main/ets/webview/Utils.ets` 新增 `setCookie` 常量 + JsHelper/ProxyJsHelper 接口；`DefaultWebview.ets` `buildJsHelper` 接入
- **API 版本**：`configCookieSync(url, value, incognito?)` 为 API 11+（满足最低 API 12）；不使用 API 14+ 的 4 参重载与 API 15+ 的 `saveCookieSync`，避免版本守卫负担
- **平台一致性**：与 Windows/macOS 的 `set_cookie` 行为对齐；`cookies()`/`delete_cookie()` 因平台限制降级，文档显式标注
- **铁律遵守**：所有 ArkTS 调用经 openharmony-ability 桥接；wry 改动限于 `cfg(target_env = "ohos")` 路径
