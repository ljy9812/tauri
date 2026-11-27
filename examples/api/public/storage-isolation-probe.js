// storage isolation probe — device-test fixture (S1-002 read-back, review
// tauri#262 item 12).
//
// Loaded as the page of a webview created by create_isolated_window — each
// such webview is given its own data_directory (webview_data_<suffix>_<seq>).
// Whether that directory actually separates storage depends on the backend:
// WebView2/webkitgtk consume it (separate storage); WKWebView and ArkWeb have
// no per-webview data path, so a same-scheme isolated window shares the main
// window's origin-scoped store (documented no-op), and the real isolation
// lever there is origin separation (useHttpsScheme → https://tauri.localhost
// vs the main window's http://tauri.localhost). The probe reports, through
// the real window.__TAURI_INTERNALS__.invoke seam (same path
// webview-viewport-probe.js validated — on OHOS this is the per-webview
// window.ipc proxy, origin-independent), what this webview's OWN localStorage
// sees:
//
//   1. sharedSeen — localStorage.getItem('sow-iso-shared'): the core.ts test
//      writes a marker under that key in the MAIN window before creating the
//      isolated windows. The test branches on the backend: on no-op
//      backends a same-scheme window must SEE it (read-back proof of the
//      no-op) while a cross-origin https-scheme window must see null (origin
//      isolation); on isolating backends both must see null.
//   2. selfWriteOk — write→read roundtrip of 'sow-iso-self' inside THIS
//      webview's own store: the isolated store is functional, not just empty.
//      The written value is this webview's label, so on shared-store
//      backends it is visible from the main window too (the test asserts
//      exactly that as the write-direction read-back).
//
// Reports fire on script eval and on window load (the poller deduplicates by
// taking the latest report per tag).
(function () {
  var statusEl = document.getElementById('status');
  function describeState(text) {
    if (statusEl) statusEl.textContent = text;
  }

  function resolveTag() {
    // Init scripts freeze per-webview metadata into __TAURI_INTERNALS__ —
    // the webview label is a unique, query-string-free probe key.
    try {
      var label = window.__TAURI_INTERNALS__ &&
        window.__TAURI_INTERNALS__.metadata &&
        window.__TAURI_INTERNALS__.metadata.currentWebview &&
        window.__TAURI_INTERNALS__.metadata.currentWebview.label;
      if (typeof label === 'string' && label.length > 0) {
        return label;
      }
    } catch (_e) { /* fall through */ }
    return 'storage-probe-unknown';
  }

  function report() {
    var tag = resolveTag();
    var sharedSeen = null;
    try {
      sharedSeen = window.localStorage.getItem('sow-iso-shared');
    } catch (_e) { /* leave null — a store that cannot even be read is a report-worthy state */ }
    var selfWriteOk = false;
    try {
      window.localStorage.setItem('sow-iso-self', tag);
      selfWriteOk = window.localStorage.getItem('sow-iso-self') === String(tag);
    } catch (_e) { /* own store not writable */ }
    describeState('storage-iso[' + tag + '] shared=' + sharedSeen +
      ' selfWrite=' + selfWriteOk);
    try {
      var internals = window.__TAURI_INTERNALS__;
      if (!internals || typeof internals.invoke !== 'function') {
        describeState('storage-iso[' + tag + ']: __TAURI_INTERNALS__.invoke missing');
        return;
      }
      // Fire-and-forget: the tracker is the durable record; a rejected invoke
      // would leave the poller timing out with the on-page state above as the
      // diagnostic.
      internals.invoke('track_storage_isolation_probe', {
        tag: tag,
        sharedSeen: sharedSeen,
        selfWriteOk: selfWriteOk
      });
    } catch (err) {
      describeState('storage-iso[' + tag + '] invoke failed: ' + String(err));
    }
  }

  report();
  window.addEventListener('load', function () { report(); });
})();
