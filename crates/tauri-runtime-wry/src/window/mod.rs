// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(all(
  any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
  ),
  not(target_env = "ohos")
))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

use crate::monitor::MonitorExt;

pub trait WindowExt {
  /// Enable or disable the window
  ///
  /// ## Platform-specific:
  ///
  /// - **Android / iOS**: Unsupported.
  /// - **OpenHarmony**: Unsupported, returns an error (#225 -- the dispatcher
  ///   fails early instead of dispatching the message: pre-fix the OHOS arm
  ///   received it and no-op'd (tao has no `set_enabled`; nothing errored
  ///   and nothing was discarded)).
  fn set_enabled(&self, enabled: bool);

  /// Whether the window is enabled or disabled.
  ///
  /// ## Platform-specific:
  ///
  /// - **Android / iOS**: Unsupported, always returns `true`.
  /// - **OpenHarmony**: Unsupported, always returns `true` (no system
  ///   enabled/disabled state to query).
  fn is_enabled(&self) -> bool;

  /// Center the window
  ///
  /// ## Platform-specific:
  ///
  /// - **Android / iOS / OpenHarmony mobile form**: Unsupported (trait
  ///   default no-op).
  /// - **OpenHarmony desktop form**: Computes the work-area center of the
  ///   window's current monitor (first available monitor as fallback) and
  ///   moves the window there via `set_outer_position`, using the same
  ///   helper as the shared `#[cfg(desktop)]` creation-time `.center()`
  ///   block (the size input differs, though: creation-time measures the
  ///   builder's target size, this path the live `outer_size`), and gated
  ///   to the same desktop form so a window created without centering is
  ///   also not centerable at runtime on mobile. No-ops when the window
  ///   geometry is not measurable yet (see the implementation comment).
  fn center(&self) {}

  /// Clears the window surface. i.e make it transparent.
  #[cfg(windows)]
  fn draw_surface(
    &self,
    surface: &mut softbuffer::Surface<
      std::sync::Arc<tao::window::Window>,
      std::sync::Arc<tao::window::Window>,
    >,
    background_color: Option<tao::window::RGBA>,
  );
}

#[cfg(any(mobile, target_env = "ohos"))]
impl WindowExt for tao::window::Window {
  fn set_enabled(&self, _: bool) {}
  fn is_enabled(&self) -> bool {
    true
  }

  /// OpenHarmony runtime centering, desktop form only — mirrors the
  /// `#[cfg(desktop)]` gate of the creation-time centering block (a
  /// mobile-form window never centers at creation, so it should not at
  /// runtime either). Android/iOS and OHOS mobile keep the trait-default
  /// no-op (method-level cfg: adding the method to this shared impl block
  /// would change the mobile surface too).
  #[cfg(all(target_env = "ohos", desktop))]
  fn center(&self) {
    // OHOS never reports a missing monitor: `current_monitor` falls back to
    // the primary (tao's OHOS window impl) and `available_monitors` always
    // yields at least the default display, so the `None` arm below is
    // defensive dead code, NOT the real early state. The real early state
    // is an unregistered window: `outer_size()` reads the default `Rect`
    // (0×0) before the window registers with the window manager, and
    // centering a 0×0 "size" puts the top-left corner on the work-area
    // center point — off by half the window's real size. The same applies
    // to a degenerate 0-sized work area. Both make centering unjudgeable;
    // keep the no-op behavior instead of moving blind.
    let monitor = self
      .current_monitor()
      .or_else(|| self.available_monitors().next());
    let Some(monitor) = monitor else {
      return;
    };
    let size = self.outer_size();
    if size.width == 0 || size.height == 0 {
      return;
    }
    let work_area = monitor.work_area();
    if work_area.size.width == 0 || work_area.size.height == 0 {
      return;
    }
    let position = calculate_window_center_position(size, monitor);
    self.set_outer_position(position);
  }
}

#[allow(dead_code)]
pub fn calculate_window_center_position(
  window_size: tao::dpi::PhysicalSize<u32>,
  target_monitor: tao::monitor::MonitorHandle,
) -> tao::dpi::PhysicalPosition<i32> {
  let work_area = target_monitor.work_area();

  tao::dpi::PhysicalPosition::new(
    (work_area.size.width as i32 - window_size.width as i32) / 2 + work_area.position.x,
    (work_area.size.height as i32 - window_size.height as i32) / 2 + work_area.position.y,
  )
}
