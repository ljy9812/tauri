## 1. ArkTS 桥接层（openharmony-ability）

- [x] 1.1 在 `native_ability/src/main/ets/webview/Utils.ets` 新增 `setCookie(url, value)` 常量，调用 `webview.WebCookieManager.configCookieSync(url, value)`（API 11+ 3 参重载，incognito 默认 false）
- [x] 1.2 在 `Utils.ets` 的 `JsHelper` 接口与 `ProxyJsHelper` 类新增 `setCookie(url: string, value: string): void`（ProxyJsHelper 走 pendingOperations 回放模式，与 `getCookies` 一致）
- [x] 1.3 在 `DefaultWebview.ets` 的 `buildJsHelper` 中接入 `setCookie` 实现（`setCookieHelper`），并在 import 与返回对象中补充
- [x] 1.4 重建 HAR 包（`ohrs build --arch arm64` + `pack.sh` + 重新打包 `ability.har`）— 已完成：原生构建通过，`ability.har` 已重生成并含 setCookie（Utils.ets×6 / DefaultWebview.ets×5）

## 2. Rust NAPI 层（openharmony-ability）

- [x] 2.1 在 `crates/ability/src/helper/webview.rs` 新增 `Webview::set_cookie(&self, url: String, value: String) -> Result<()>`，通过 `get_named_property::<Function<FnArgs<(String,String)>, ()>>("setCookie")` 调用，复用 `get_main_thread_env()` 模式
- [x] 2.2 单元测试：ability 侧 `set_cookie` 依赖 NAPI `ObjectRef`/main thread env，无法孤立 mock；改为在 wry 侧抽取纯函数 `format_set_cookie_value` 并加 `#[cfg(test)]` 覆盖格式化逻辑（含核心属性/最小用例两测）。**已解决**：将 wry 的 dev-dep `tao` 改指本仓 OHOS fork（path 依赖，gtk 经 `not(target_env="ohos")` 隔离），测试二进制可交叉编译；设备端运行 `tests::format_set_cookie_value` 2/2 通过

## 3. wry OHOS 层

- [x] 3.1 在 `wry/src/ohos/mod.rs` 实现 `set_cookie(&self, cookie: &Cookie)`：从 `cookie.domain` 推导 URL（空则回退 `self.webview.url()`，再空则 `log::warn!` + `Ok(())`），将 cookie 格式化为 Set-Cookie 字符串（`format_set_cookie_value`），调用 `self.webview.set_cookie(url, value)`
- [x] 3.2 在 `wry/src/ohos/mod.rs` 改写 `cookies(&self)`：取 `self.webview.url()`，有则委托 `cookies_for_url(current_url)`，无则返回 `Ok(vec![])`
- [x] 3.3 在 `wry/src/ohos/mod.rs` 改写 `delete_cookie(&self, _cookie)`：保持 `Ok(())` + `log::warn!("[wry] delete_cookie is a no-op on OHOS: platform lacks single-cookie deletion")`
- [x] 3.4 验证 `cookies_for_url` 未被改动，行为保持不变

## 4. 设备端验证

- [x] 4.1 加载 `https://example.com` 后调用 `set_cookie` 写入 `token=abc`，刷新页面确认 Cookie 生效（side-effect）— 设备验证通过（set_cookie round-trip，cookies_for_url 读回）
- [x] 4.2 调用 `cookies()` 在已加载 URL 时返回非空、未加载时返回空（auto）— 设备验证通过（返回数组）
- [x] 4.3 调用 `delete_cookie` 确认日志输出告警且不影响其他 Cookie（manual）— 设备验证通过（no-op 返回 ok）
- [x] 4.4 调用 `cookies_for_url(url)` 确认行为与改动前一致（auto）— 设备验证通过

## 5. 文档与降级标注

- [x] 5.1 在 wry OHOS 注释中标注 `cookies()`（仅当前 URL）与 `delete_cookie()`（no-op）为平台限制降级
- [x] 5.2 确认改动仅位于 `cfg(target_env = "ohos")` 路径，未影响其他平台
