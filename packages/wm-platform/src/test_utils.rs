//! Utilities for testing.
//!
//! Available via the `test_utils` Cargo feature.
use std::sync::{atomic::AtomicBool, Arc};

use crate::{
  Dispatcher, Display, NativeWindow, NativeWindowImpl, Rect, RectDelta,
  WindowId,
};

/// Tracks method calls on a `TrackedNativeWindow`.
///
/// Shared between the mock window and the test via `Arc`, allowing tests
/// to verify which methods were called after exercising the code under
/// test.
#[derive(Debug, Default)]
pub struct CallTracker {
  maximize_called: std::sync::atomic::AtomicU32,
  minimize_called: std::sync::atomic::AtomicU32,
  set_frame_called: std::sync::atomic::AtomicU32,
}

impl CallTracker {
  /// Returns the number of times `maximize()` was called.
  pub fn maximize_called(&self) -> u32 {
    self
      .maximize_called
      .load(std::sync::atomic::Ordering::SeqCst)
  }

  /// Returns the number of times `minimize()` was called.
  pub fn minimize_called(&self) -> u32 {
    self
      .minimize_called
      .load(std::sync::atomic::Ordering::SeqCst)
  }

  /// Returns the number of times `set_frame()` was called.
  pub fn set_frame_called(&self) -> u32 {
    self
      .set_frame_called
      .load(std::sync::atomic::Ordering::SeqCst)
  }
}

/// A mock `NativeWindow` that tracks method calls via a shared
/// `CallTracker`.
///
/// Use `NativeWindowImpl::mock_with_tracker()` to create an instance and
/// obtain both the `Arc<dyn NativeWindow>` and the `Arc<CallTracker>`.
pub struct TrackedNativeWindow {
  id: WindowId,
  title: String,
  tracker: Arc<CallTracker>,
}

impl TrackedNativeWindow {
  fn new(id: WindowId, title: String, tracker: Arc<CallTracker>) -> Self {
    Self { id, title, tracker }
  }
}

impl NativeWindow for TrackedNativeWindow {
  fn id(&self) -> WindowId {
    self.id
  }

  fn title(&self) -> crate::Result<String> {
    Ok(self.title.clone())
  }

  fn process_name(&self) -> crate::Result<String> {
    Ok(String::new())
  }

  fn frame(&self) -> crate::Result<Rect> {
    Ok(Rect::from_xy(0, 0, 100, 100))
  }

  fn position(&self) -> crate::Result<(f64, f64)> {
    Ok((0.0, 0.0))
  }

  fn size(&self) -> crate::Result<(f64, f64)> {
    Ok((100.0, 100.0))
  }

  fn is_valid(&self) -> bool {
    true
  }

  fn is_visible(&self) -> crate::Result<bool> {
    Ok(true)
  }

  fn is_minimized(&self) -> crate::Result<bool> {
    Ok(false)
  }

  fn is_maximized(&self) -> crate::Result<bool> {
    Ok(false)
  }

  fn is_resizable(&self) -> crate::Result<bool> {
    Ok(true)
  }

  fn is_desktop_window(&self) -> crate::Result<bool> {
    Ok(false)
  }

  fn set_frame(&self, _rect: &Rect) -> crate::Result<()> {
    self
      .tracker
      .set_frame_called
      .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ok(())
  }

  fn resize(&self, _width: i32, _height: i32) -> crate::Result<()> {
    Ok(())
  }

  fn reposition(&self, _x: i32, _y: i32) -> crate::Result<()> {
    Ok(())
  }

  fn minimize(&self) -> crate::Result<()> {
    self
      .tracker
      .minimize_called
      .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ok(())
  }

  fn maximize(&self) -> crate::Result<()> {
    self
      .tracker
      .maximize_called
      .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ok(())
  }

  fn focus(&self) -> crate::Result<()> {
    Ok(())
  }

  fn close(&self) -> crate::Result<()> {
    Ok(())
  }

  #[cfg(target_os = "windows")]
  fn class_name(&self) -> crate::Result<String> {
    Ok(String::new())
  }

  #[cfg(target_os = "windows")]
  fn shadow_borders(&self) -> crate::Result<RectDelta> {
    Ok(RectDelta::zero())
  }

  #[cfg(target_os = "windows")]
  fn as_windows_ext(
    &self,
  ) -> crate::Result<&dyn crate::NativeWindowWindowsExt> {
    Err(crate::Error::Platform(
      "TrackedNativeWindow does not implement WindowsExt".into(),
    ))
  }

  #[cfg(target_os = "macos")]
  fn as_macos_ext(
    &self,
  ) -> crate::Result<&dyn crate::NativeWindowExtMacOs> {
    Err(crate::Error::Platform(
      "TrackedNativeWindow does not implement MacOsExt".into(),
    ))
  }
}

impl std::fmt::Debug for TrackedNativeWindow {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TrackedNativeWindow")
      .field("id", &self.id)
      .field("title", &self.title)
      .finish()
  }
}

/// A simple mock `NativeWindow` with configurable properties.
///
/// Use `NativeWindowImpl::mock()` or `NativeWindowImpl::mock_with()` to
/// create instances.
pub struct MockNativeWindow {
  id: WindowId,
  title: String,
  process_name: String,
  frame: Rect,
  position: (f64, f64),
  size: (f64, f64),
  is_valid: bool,
  is_visible: bool,
  is_minimized: bool,
  is_maximized: bool,
  is_resizable: bool,
  is_desktop_window: bool,
}

impl MockNativeWindow {
  #[must_use]
  pub fn new() -> Self {
    Self {
      id: WindowId(0),
      title: String::new(),
      process_name: String::new(),
      frame: Rect::from_xy(0, 0, 100, 100),
      position: (0.0, 0.0),
      size: (100.0, 100.0),
      is_valid: true,
      is_visible: true,
      is_minimized: false,
      is_maximized: false,
      is_resizable: true,
      is_desktop_window: false,
    }
  }

  #[must_use]
  pub fn with_id(mut self, id: WindowId) -> Self {
    self.id = id;
    self
  }

  #[must_use]
  pub fn with_title(mut self, title: String) -> Self {
    self.title = title;
    self
  }

  #[must_use]
  pub fn with_frame(mut self, frame: Rect) -> Self {
    self.frame = frame;
    self
  }
}

impl Default for MockNativeWindow {
  fn default() -> Self {
    Self::new()
  }
}

impl NativeWindow for MockNativeWindow {
  fn id(&self) -> WindowId {
    self.id
  }

  fn title(&self) -> crate::Result<String> {
    Ok(self.title.clone())
  }

  fn process_name(&self) -> crate::Result<String> {
    Ok(self.process_name.clone())
  }

  fn frame(&self) -> crate::Result<Rect> {
    Ok(self.frame.clone())
  }

  fn position(&self) -> crate::Result<(f64, f64)> {
    Ok(self.position)
  }

  fn size(&self) -> crate::Result<(f64, f64)> {
    Ok(self.size)
  }

  fn is_valid(&self) -> bool {
    self.is_valid
  }

  fn is_visible(&self) -> crate::Result<bool> {
    Ok(self.is_visible)
  }

  fn is_minimized(&self) -> crate::Result<bool> {
    Ok(self.is_minimized)
  }

  fn is_maximized(&self) -> crate::Result<bool> {
    Ok(self.is_maximized)
  }

  fn is_resizable(&self) -> crate::Result<bool> {
    Ok(self.is_resizable)
  }

  fn is_desktop_window(&self) -> crate::Result<bool> {
    Ok(self.is_desktop_window)
  }

  fn set_frame(&self, _rect: &Rect) -> crate::Result<()> {
    Ok(())
  }

  fn resize(&self, _width: i32, _height: i32) -> crate::Result<()> {
    Ok(())
  }

  fn reposition(&self, _x: i32, _y: i32) -> crate::Result<()> {
    Ok(())
  }

  fn minimize(&self) -> crate::Result<()> {
    Ok(())
  }

  fn maximize(&self) -> crate::Result<()> {
    Ok(())
  }

  fn focus(&self) -> crate::Result<()> {
    Ok(())
  }

  fn close(&self) -> crate::Result<()> {
    Ok(())
  }

  #[cfg(target_os = "windows")]
  fn class_name(&self) -> crate::Result<String> {
    Ok(String::new())
  }

  #[cfg(target_os = "windows")]
  fn shadow_borders(&self) -> crate::Result<RectDelta> {
    Ok(RectDelta::zero())
  }

  #[cfg(target_os = "windows")]
  fn as_windows_ext(
    &self,
  ) -> crate::Result<&dyn crate::NativeWindowWindowsExt> {
    Err(crate::Error::Platform(
      "Mock does not implement WindowsExt".into(),
    ))
  }

  #[cfg(target_os = "macos")]
  fn as_macos_ext(
    &self,
  ) -> crate::Result<&dyn crate::NativeWindowExtMacOs> {
    Err(crate::Error::Platform(
      "Mock does not implement MacOsExt".into(),
    ))
  }
}

impl std::fmt::Debug for MockNativeWindow {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MockNativeWindow")
      .field("id", &self.id)
      .field("title", &self.title)
      .finish()
  }
}

impl Dispatcher {
  /// Creates a mock `Dispatcher` for use in tests.
  ///
  /// Calling any methods on the mock is undefined behavior and may panic.
  #[must_use]
  pub fn mock() -> Self {
    Self::new(None, Arc::new(AtomicBool::new(false)))
  }
}

impl NativeWindowImpl {
  /// Creates a mock `NativeWindow` for use in tests.
  ///
  /// The mock returns default values for all methods. Use `mock_with` for
  /// a mock with a specific ID and title, or `mock_with_tracker` for a
  /// mock that tracks method calls.
  #[must_use]
  pub fn mock() -> Arc<dyn NativeWindow> {
    Arc::new(MockNativeWindow::new()) as Arc<dyn NativeWindow>
  }
}

impl NativeWindowImpl {
  /// Creates a mock `NativeWindow` with the given ID and title for use in
  /// tests.
  #[must_use]
  pub fn mock_with(id: WindowId, title: &str) -> Arc<dyn NativeWindow> {
    Arc::new(
      MockNativeWindow::new()
        .with_id(id)
        .with_title(title.to_string()),
    ) as Arc<dyn NativeWindow>
  }
}

impl NativeWindowImpl {
  /// Creates a mock `NativeWindow` with call tracking.
  ///
  /// Returns a tuple of `(Arc<CallTracker>, Arc<dyn NativeWindow>)` where
  /// the `CallTracker` can be used after exercising the code under test to
  /// verify which methods were called and how many times.
  #[must_use]
  pub fn mock_with_tracker(
    id: WindowId,
    title: &str,
  ) -> (Arc<CallTracker>, Arc<dyn NativeWindow>) {
    let tracker = Arc::new(CallTracker::default());
    let window = Arc::new(TrackedNativeWindow::new(
      id,
      title.to_string(),
      Arc::clone(&tracker),
    )) as Arc<dyn NativeWindow>;
    (tracker, window)
  }
}

impl Display {
  /// Creates a mock `Display` for use in tests.
  ///
  /// Calling any methods on the mock is undefined behavior and may panic.
  #[must_use]
  pub fn mock() -> Self {
    Self {
      #[cfg(target_os = "windows")]
      inner: crate::platform_impl::Display::new(0),
      #[cfg(target_os = "macos")]
      #[allow(invalid_value)]
      inner: unsafe { std::mem::zeroed() },
    }
  }
}
