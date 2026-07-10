# webview-cookie-mgmt Specification

## Purpose
TBD - created by archiving change p1-webview-cookie. Update Purpose after archive.
## Requirements
### Requirement: Set a single cookie
The system SHALL allow setting a single cookie on OHOS by delegating to `WebCookieManager.configCookieSync`, deriving the target URL from the cookie's domain and formatting the cookie as a Set-Cookie string.

#### Scenario: Set cookie with domain
- **WHEN** `set_cookie(cookie)` is called with a cookie whose `domain` is `example.com`, `name`=`token`, `value`=`abc`
- **THEN** the system calls `configCookieSync("https://example.com", "token=abc; Domain=example.com; Path=/")` and returns `Ok(())`

#### Scenario: Set cookie without domain falls back to current URL
- **WHEN** `set_cookie(cookie)` is called with a cookie whose `domain` is empty and the webview current URL is `https://foo.com`
- **THEN** the system uses `https://foo.com` as the target URL

#### Scenario: Set cookie with no domain and no current URL
- **WHEN** `set_cookie(cookie)` is called with an empty domain and the current URL cannot be obtained
- **THEN** the system logs a warning and returns `Ok(())` without calling the ArkTS API

### Requirement: Get cookies best-effort
Because OHOS `WebCookieManager` has no API to enumerate all cookies across all URLs, `cookies()` SHALL return cookies for the webview's current URL as a best-effort degradation, and SHALL return an empty vector when no current URL is available.

#### Scenario: Get cookies with a loaded URL
- **WHEN** `cookies()` is called and the webview current URL is `https://example.com`
- **THEN** the system returns the parsed cookies for that URL (same result as `cookies_for_url("https://example.com")`)

#### Scenario: Get cookies with no URL
- **WHEN** `cookies()` is called and the current URL is empty or cannot be obtained
- **THEN** the system returns an empty `Vec<Cookie>`

### Requirement: Delete single cookie is a no-op
Because OHOS `WebCookieManager` provides no single-cookie deletion (only `clearAllCookies` / `clearSessionCookie`), `delete_cookie(cookie)` SHALL be a no-op that returns `Ok(())` and emits a warning. It SHALL NOT call `clearAllCookies` (which would incorrectly delete all cookies).

#### Scenario: Delete cookie logs warning
- **WHEN** `delete_cookie(cookie)` is called
- **THEN** the system emits `log::warn!` indicating the platform lacks single-cookie deletion and returns `Ok(())` without modifying any cookie store

### Requirement: cookies_for_url remains available
The existing `cookies_for_url(url)` behavior SHALL remain unchanged and continue to return parsed cookies for the given URL via `WebCookieManager.fetchCookieSync`.

#### Scenario: cookies_for_url unchanged
- **WHEN** `cookies_for_url("https://example.com")` is called
- **THEN** the system returns the same parsed cookies as before this change (no behavioral change)

### Requirement: API version safety
The implementation SHALL use only `WebCookieManager` APIs available at API version 11+ (`fetchCookieSync`, `configCookieSync(url, value, incognito?)`, `clearAllCookies*`) and SHALL NOT use API 14+ (`configCookieSync` 4-arg) or API 15+ (`saveCookieSync`) overloads without a version guard, to remain compatible with the minimum supported API 12.

#### Scenario: No API 14+ overload used
- **WHEN** the ArkTS bridge sets a cookie
- **THEN** it calls the 3-argument `configCookieSync(url, value, incognito?)` form, not the 4-argument API 14+ form

