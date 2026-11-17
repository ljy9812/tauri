// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_utils::{config::BundleType, Theme};

use crate::{
  command,
  plugin::{Builder, TauriPlugin},
  AppHandle, Manager, ResourceId, Runtime, Webview,
};

#[command(root = "crate")]
pub fn version<R: Runtime>(app: AppHandle<R>) -> String {
  app.package_info().version.to_string()
}

#[command(root = "crate")]
pub fn name<R: Runtime>(app: AppHandle<R>) -> String {
  app.package_info().name.clone()
}

#[command(root = "crate")]
pub fn tauri_version() -> &'static str {
  crate::VERSION
}

#[command(root = "crate")]
pub fn identifier<R: Runtime>(app: AppHandle<R>) -> String {
  app.config().identifier.clone()
}

#[command(root = "crate")]
#[allow(unused_variables)]
pub fn app_show<R: Runtime>(app: AppHandle<R>) -> crate::Result<()> {
  #[cfg(target_os = "macos")]
  app.show()?;
  Ok(())
}

#[command(root = "crate")]
#[allow(unused_variables)]
pub fn app_hide<R: Runtime>(app: AppHandle<R>) -> crate::Result<()> {
  #[cfg(target_os = "macos")]
  app.hide()?;
  Ok(())
}

#[command(root = "crate")]
#[allow(unused_variables)]
pub async fn fetch_data_store_identifiers<R: Runtime>(
  app: AppHandle<R>,
) -> crate::Result<Vec<[u8; 16]>> {
  #[cfg(target_vendor = "apple")]
  return app.fetch_data_store_identifiers().await;
  #[cfg(not(target_vendor = "apple"))]
  return Ok(Vec::new());
}

#[command(root = "crate")]
#[allow(unused_variables)]
pub async fn remove_data_store<R: Runtime>(app: AppHandle<R>, uuid: [u8; 16]) -> crate::Result<()> {
  #[cfg(target_vendor = "apple")]
  app.remove_data_store(uuid).await?;
  #[cfg(not(target_vendor = "apple"))]
  let _ = uuid;
  Ok(())
}

#[command(root = "crate")]
pub fn default_window_icon<R: Runtime>(
  webview: Webview<R>,
  app: AppHandle<R>,
) -> Option<ResourceId> {
  app.default_window_icon().cloned().map(|icon| {
    let mut resources_table = webview.resources_table();
    resources_table.add(icon.to_owned())
  })
}

/// How long the `set_app_theme` command waits for the runtime receipt before
/// falling back to the legacy fire-and-forget success.
///
/// Production: fixed 5s. Tests inject a short value through
/// `TEST_SET_APP_THEME_RECEIPT_TIMEOUT_MS` (`#[cfg(test)]` only — the
/// production path is not injectable); an injection that does not take effect
/// or a test that really waits 5s is an implementation defect.
#[cfg(not(test))]
fn set_app_theme_receipt_timeout() -> std::time::Duration {
  std::time::Duration::from_secs(5)
}

#[cfg(test)]
thread_local! {
  /// Test-injectable receipt timeout in milliseconds. Thread-local because
  /// `#[tokio::test]` futures run on a single current-thread runtime, so each
  /// test observes its own value without cross-test interference.
  static TEST_SET_APP_THEME_RECEIPT_TIMEOUT_MS: std::cell::Cell<u64> =
    const { std::cell::Cell::new(5_000) };
}

#[cfg(test)]
fn set_app_theme_receipt_timeout() -> std::time::Duration {
  std::time::Duration::from_millis(TEST_SET_APP_THEME_RECEIPT_TIMEOUT_MS.with(std::cell::Cell::get))
}

/// Dispatches the receipted theme change to the runtime.
#[cfg(not(test))]
fn dispatch_theme_with_receipt<R: Runtime>(
  app: &AppHandle<R>,
  theme: Option<Theme>,
  receipt: Box<dyn FnOnce(bool) + Send + 'static>,
) -> crate::Result<()> {
  app.set_theme_with_receipt(theme, receipt)
}

/// Test-only alias for the receipt closure (same shape as the bare `Box` the
/// runtime traits and the wry message carry).
#[cfg(test)]
type TestReceipt = Box<dyn FnOnce(bool) + Send + 'static>;

#[cfg(test)]
thread_local! {
  /// Records the verdict the runtime path delivered to the receipt, so the
  /// pass-through test can tell "fired with `true`" apart from "receipt
  /// dropped without firing" — both resolve the command to `Ok`, only the
  /// verdict is a precise observable. Fresh `None` per test thread.
  static TEST_LAST_RECEIPT_VERDICT: std::cell::RefCell<Option<bool>> =
    const { std::cell::RefCell::new(None) };

  /// Receipts parked by the `Hold` mode. Parking keeps the sender alive (like
  /// an undrained tao completion queue) so the command hits its TIMEOUT
  /// branch; plainly dropping the receipt would instead resolve the receiver
  /// instantly with a `RecvError` and dodge the timeout path the test must
  /// cover (and the delete-timeout mutation must turn red).
  static TEST_PARKED_RECEIPTS: std::cell::RefCell<Vec<TestReceipt>> =
    const { std::cell::RefCell::new(Vec::new()) };
}

/// Test injection seam.
///
/// `Real` goes through the real runtime dispatch, whose `RuntimeHandle`
/// default implementation applies the theme and fires the receipt with `true`
/// — this path must stay wired to the real trait body because the
/// delete-default-receipt mutation has to turn the pass-through test red.
/// `Fail`/`Hold` simulate the OHOS runtime delivering a failure verdict /
/// starving the receipt; the simulation lives here and not in a mock
/// `RuntimeHandle` override because an override would shadow the very trait
/// default body the `Real` path exercises.
#[cfg(test)]
fn dispatch_theme_with_receipt<R: Runtime>(
  app: &AppHandle<R>,
  theme: Option<Theme>,
  receipt: Box<dyn FnOnce(bool) + Send + 'static>,
) -> crate::Result<()> {
  match crate::test::theme_receipt_mode() {
    crate::test::ThemeReceiptMode::Fail => {
      receipt(false);
      Ok(())
    }
    crate::test::ThemeReceiptMode::Hold => {
      TEST_PARKED_RECEIPTS.with(|cell| cell.borrow_mut().push(receipt));
      Ok(())
    }
    crate::test::ThemeReceiptMode::Real => {
      let wrapped: Box<dyn FnOnce(bool) + Send + 'static> = Box::new(move |ok| {
        TEST_LAST_RECEIPT_VERDICT.with(|cell| *cell.borrow_mut() = Some(ok));
        receipt(ok);
      });
      app.set_theme_with_receipt(theme, wrapped)
    }
  }
}

#[command(root = "crate")]
pub async fn set_app_theme<R: Runtime>(
  app: AppHandle<R>,
  theme: Option<Theme>,
) -> Result<(), String> {
  let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
  let receipt: Box<dyn FnOnce(bool) + Send + 'static> = Box::new(move |ok| {
    // The awaiting side may already have timed out and dropped its receiver —
    // a normal, non-fatal outcome (mirrors the tao ThemeReceipt contract).
    let _ = tx.send(ok);
  });

  if let Err(err) = dispatch_theme_with_receipt(&app, theme, receipt) {
    log::warn!("failed to dispatch set_app_theme to the runtime: {err}; keeping the legacy fire-and-forget success semantics");
    return Ok(());
  }

  match tokio::time::timeout(set_app_theme_receipt_timeout(), rx).await {
    // The deferred ArkTS tick applied the color mode change.
    Ok(Ok(true)) => Ok(()),
    // The deferred ArkTS tick failed and the runtime already rolled the
    // theme override back to follow-system; surface a real error to the JS
    // caller instead of the old unconditional Ok.
    Ok(Ok(false)) => Err(
      "setColorMode failed on the deferred ArkTS tick — the theme override rolled back to follow-system"
        .to_string(),
    ),
    // The receipt was dropped without a verdict (the closure always sends,
    // so this should not happen). Keep the legacy success semantics.
    Ok(Err(_)) => {
      log::warn!("set_app_theme receipt channel dropped without a verdict");
      Ok(())
    }
    // Timed out: stale-HAR / early-startup cases where no deferred
    // completion will ever be drained. Keep the legacy fire-and-forget
    // semantics instead of hanging the JS caller.
    Err(_) => {
      log::warn!("set_app_theme receipt timed out without a verdict");
      Ok(())
    }
  }
}

#[command(root = "crate")]
pub async fn set_dock_visibility<R: Runtime>(
  app: AppHandle<R>,
  visible: bool,
) -> crate::Result<()> {
  #[cfg(target_os = "macos")]
  {
    let mut focused_window = None;
    for window in app.manager.windows().into_values() {
      if window.is_focused().unwrap_or_default() {
        focused_window.replace(window);
        break;
      }
    }

    app.set_dock_visibility(visible)?;

    // retain focus
    if let Some(focused_window) = focused_window {
      let _ = focused_window.set_focus();
    }
  }
  #[cfg(not(target_os = "macos"))]
  let (_app, _visible) = (app, visible);
  Ok(())
}

#[command(root = "crate")]
pub fn bundle_type() -> Option<BundleType> {
  tauri_utils::platform::bundle_type()
}

#[command(root = "crate")]
pub fn supports_multiple_windows<R: Runtime>(app: AppHandle<R>) -> bool {
  app.supports_multiple_windows()
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("app")
    .invoke_handler(crate::generate_handler![
      #![plugin(app)]
      version,
      name,
      tauri_version,
      identifier,
      app_show,
      app_hide,
      fetch_data_store_identifiers,
      remove_data_store,
      default_window_icon,
      set_app_theme,
      set_dock_visibility,
      bundle_type,
      supports_multiple_windows,
    ])
    .setup(|_app, _api| {
      #[cfg(target_os = "android")]
      {
        let handle = _api.register_android_plugin("app.tauri", "AppPlugin")?;
        _app.manage(AppPlugin(handle));
      }
      Ok(())
    })
    .build()
}

#[cfg(target_os = "android")]
pub(crate) struct AppPlugin<R: Runtime>(pub crate::plugin::PluginHandle<R>);

#[cfg(test)]
mod tests {
  use std::time::{Duration, Instant};

  use tauri_utils::Theme;

  use super::{set_app_theme, TEST_LAST_RECEIPT_VERDICT, TEST_SET_APP_THEME_RECEIPT_TIMEOUT_MS};
  use crate::{
    test::{mock_app, set_theme_receipt_mode, MockRuntime, ThemeReceiptMode},
    App, Manager,
  };

  fn receipt_handle(app: &App<MockRuntime>) -> crate::test::MockRuntimeHandle {
    app.app_handle().runtime_handle.clone()
  }

  #[tokio::test]
  async fn set_app_theme_surfaces_deferred_failure_as_err() {
    set_theme_receipt_mode(ThemeReceiptMode::Fail);
    TEST_SET_APP_THEME_RECEIPT_TIMEOUT_MS.set(50);

    let app = mock_app();
    let result = set_app_theme(app.app_handle().clone(), Some(Theme::Dark)).await;

    let err = result.expect_err("a failed deferred setColorMode must reject");
    assert!(err.contains("failed"), "must state the failure: {err}");
    assert!(err.contains("roll"), "must state the rollback: {err}");
  }

  #[tokio::test]
  async fn set_app_theme_falls_back_to_ok_when_receipt_never_fires() {
    set_theme_receipt_mode(ThemeReceiptMode::Hold);
    TEST_SET_APP_THEME_RECEIPT_TIMEOUT_MS.set(50);

    let app = mock_app();
    let started = Instant::now();
    let result = set_app_theme(app.app_handle().clone(), Some(Theme::Dark)).await;
    let elapsed = started.elapsed();

    assert!(
      result.is_ok(),
      "the timeout fallback must keep the legacy Ok semantics: {result:?}"
    );
    assert!(
      elapsed < Duration::from_secs(1),
      "the injected short timeout must take effect (took {elapsed:?}); \
       actually waiting the 5s default means the injection point is broken"
    );
  }

  /// The `Real` mode dispatches through `AppHandle::set_theme_with_receipt`
  /// into the real `RuntimeHandle` default body (`set_theme` +
  /// `receipt(true)`): deleting the default `receipt(true)` must turn this
  /// test red via the recorded-verdict guard below (a dropped receipt would
  /// resolve the command to `Ok` through the `RecvError` fallback, so the
  /// command's own result cannot tell the two apart).
  #[tokio::test]
  async fn set_app_theme_resolves_ok_when_receipt_reports_success() {
    set_theme_receipt_mode(ThemeReceiptMode::Real);
    TEST_SET_APP_THEME_RECEIPT_TIMEOUT_MS.set(800);

    let app = mock_app();
    let started = Instant::now();
    let result = set_app_theme(app.app_handle().clone(), Some(Theme::Dark)).await;
    let elapsed = started.elapsed();

    assert!(
      result.is_ok(),
      "a successful receipt must resolve Ok: {result:?}"
    );
    assert_eq!(
      TEST_LAST_RECEIPT_VERDICT.with(|cell| *cell.borrow()),
      Some(true),
      "the runtime path must fire the receipt with the success verdict"
    );
    assert!(
      elapsed < Duration::from_millis(400),
      "the receipt must fire immediately (took {elapsed:?}); \
       a ~800ms/5s wait means the timeout injection is broken"
    );
    assert_eq!(
      receipt_handle(&app).set_theme_calls(),
      1,
      "the receipted path must still dispatch the theme change to the runtime"
    );
  }
}
