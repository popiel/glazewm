//! Utilities for testing.
//!
//! Available via the `test_utils` Cargo feature.
use std::sync::{atomic::AtomicBool, Arc};

use bon::bon;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE;

use crate::{
  Color, CornerStyle, Delta, Dispatcher, Display, NativeWindow,
  NativeWindowImpl, OpacityValue, Point, Rect, RectDelta, WindowId,
  WindowZOrder,
};

/// A method on a platform type that was called.
///
/// Used as a discriminator for [`PlatformCall`] to filter calls by method
/// name without matching on arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformMethod {
  // NativeWindow methods
  Id,
  Title,
  ProcessName,
  Frame,
  Position,
  Size,
  IsValid,
  IsVisible,
  IsMinimized,
  IsMaximized,
  IsResizable,
  IsDesktopWindow,
  SetFrame,
  Resize,
  Reposition,
  Minimize,
  Maximize,
  Focus,
  Close,
  // NativeWindowWindowsExt methods
  #[cfg(target_os = "windows")]
  ClassName,
  #[cfg(target_os = "windows")]
  FrameWithShadows,
  #[cfg(target_os = "windows")]
  ShadowBorders,
  #[cfg(target_os = "windows")]
  HasOwnerWindow,
  #[cfg(target_os = "windows")]
  HasWindowStyle,
  #[cfg(target_os = "windows")]
  HasWindowStyleEx,
  #[cfg(target_os = "windows")]
  SetWindowPos,
  #[cfg(target_os = "windows")]
  Show,
  #[cfg(target_os = "windows")]
  Hide,
  #[cfg(target_os = "windows")]
  Restore,
  #[cfg(target_os = "windows")]
  SetCloaked,
  #[cfg(target_os = "windows")]
  MarkFullscreen,
  #[cfg(target_os = "windows")]
  SetTaskbarVisibility,
  #[cfg(target_os = "windows")]
  AddWindowStyleEx,
  #[cfg(target_os = "windows")]
  SetZOrder,
  #[cfg(target_os = "windows")]
  SetTitleBarVisibility,
  #[cfg(target_os = "windows")]
  SetBorderColor,
  #[cfg(target_os = "windows")]
  SetCornerStyle,
  #[cfg(target_os = "windows")]
  SetTransparency,
  #[cfg(target_os = "windows")]
  AdjustTransparency,
  // NativeWindowExtMacOs methods
  #[cfg(target_os = "macos")]
  BundleId,
  #[cfg(target_os = "macos")]
  Role,
  #[cfg(target_os = "macos")]
  Subrole,
  #[cfg(target_os = "macos")]
  IsModal,
  #[cfg(target_os = "macos")]
  IsMain,
  // Dispatcher methods
  ResetFocus,
  CursorPosition,
  SetCursorPosition,
}

/// A recorded call to a platform method.
///
/// Each variant corresponds to a method on `NativeWindow`,
/// `NativeWindowWindowsExt`, or `NativeWindowExtMacOs`. The `id` field
/// identifies which window instance the call was made on. Arguments and
/// return values are stored by value.
#[derive(Debug, Clone)]
pub enum PlatformCall {
  // --- NativeWindow methods ---
  Id {
    id: WindowId,
    result: WindowId,
  },
  Title {
    id: WindowId,
    result: String,
  },
  ProcessName {
    id: WindowId,
    result: String,
  },
  Frame {
    id: WindowId,
    result: Rect,
  },
  Position {
    id: WindowId,
    result: (f64, f64),
  },
  Size {
    id: WindowId,
    result: (f64, f64),
  },
  IsValid {
    id: WindowId,
    result: bool,
  },
  IsVisible {
    id: WindowId,
    result: bool,
  },
  IsMinimized {
    id: WindowId,
    result: bool,
  },
  IsMaximized {
    id: WindowId,
    result: bool,
  },
  IsResizable {
    id: WindowId,
    result: bool,
  },
  IsDesktopWindow {
    id: WindowId,
    result: bool,
  },
  SetFrame {
    id: WindowId,
    rect: Rect,
  },
  Resize {
    id: WindowId,
    width: i32,
    height: i32,
  },
  Reposition {
    id: WindowId,
    x: i32,
    y: i32,
  },
  Minimize {
    id: WindowId,
  },
  Maximize {
    id: WindowId,
  },
  Focus {
    id: WindowId,
  },
  Close {
    id: WindowId,
  },

  // --- NativeWindowWindowsExt methods ---
  #[cfg(target_os = "windows")]
  ClassName {
    id: WindowId,
    result: String,
  },
  #[cfg(target_os = "windows")]
  FrameWithShadows {
    id: WindowId,
    result: Rect,
  },
  #[cfg(target_os = "windows")]
  ShadowBorders {
    id: WindowId,
    result: RectDelta,
  },
  #[cfg(target_os = "windows")]
  HasOwnerWindow {
    id: WindowId,
    result: bool,
  },
  #[cfg(target_os = "windows")]
  HasWindowStyle {
    id: WindowId,
    style: WINDOW_STYLE,
    result: bool,
  },
  #[cfg(target_os = "windows")]
  HasWindowStyleEx {
    id: WindowId,
    style: WINDOW_EX_STYLE,
    result: bool,
  },
  #[cfg(target_os = "windows")]
  SetWindowPos {
    id: WindowId,
    z_order: WindowZOrder,
    rect: Rect,
    flags: SET_WINDOW_POS_FLAGS,
  },
  #[cfg(target_os = "windows")]
  Show {
    id: WindowId,
  },
  #[cfg(target_os = "windows")]
  Hide {
    id: WindowId,
  },
  #[cfg(target_os = "windows")]
  Restore {
    id: WindowId,
    outer_frame: Option<Rect>,
  },
  #[cfg(target_os = "windows")]
  SetCloaked {
    id: WindowId,
    cloaked: bool,
  },
  #[cfg(target_os = "windows")]
  MarkFullscreen {
    id: WindowId,
    fullscreen: bool,
  },
  #[cfg(target_os = "windows")]
  SetTaskbarVisibility {
    id: WindowId,
    visible: bool,
  },
  #[cfg(target_os = "windows")]
  AddWindowStyleEx {
    id: WindowId,
    style: WINDOW_EX_STYLE,
  },
  #[cfg(target_os = "windows")]
  SetZOrder {
    id: WindowId,
    zorder: WindowZOrder,
  },
  #[cfg(target_os = "windows")]
  SetTitleBarVisibility {
    id: WindowId,
    visible: bool,
  },
  #[cfg(target_os = "windows")]
  SetBorderColor {
    id: WindowId,
    color: Option<Color>,
  },
  #[cfg(target_os = "windows")]
  SetCornerStyle {
    id: WindowId,
    corner_style: CornerStyle,
  },
  #[cfg(target_os = "windows")]
  SetTransparency {
    id: WindowId,
    opacity_value: OpacityValue,
  },
  #[cfg(target_os = "windows")]
  AdjustTransparency {
    id: WindowId,
    opacity_delta: Delta<OpacityValue>,
  },

  // --- NativeWindowExtMacOs methods ---
  #[cfg(target_os = "macos")]
  BundleId {
    id: WindowId,
    result: Option<String>,
  },
  #[cfg(target_os = "macos")]
  Role {
    id: WindowId,
    result: String,
  },
  #[cfg(target_os = "macos")]
  Subrole {
    id: WindowId,
    result: String,
  },
  #[cfg(target_os = "macos")]
  IsModal {
    id: WindowId,
    result: bool,
  },
  #[cfg(target_os = "macos")]
  IsMain {
    id: WindowId,
    result: bool,
  },

  // --- Dispatcher methods ---
  ResetFocus,
  CursorPosition {
    result: Point,
  },
  SetCursorPosition {
    point: Point,
  },
}

impl PlatformCall {
  /// Returns the `WindowId` of the window instance this call was made on.
  ///
  /// Returns `None` for `Dispatcher` calls, which are not associated with
  /// a specific window.
  ///
  /// # Panics
  ///
  /// Panics if the mutex is poisoned.
  #[must_use]
  pub fn id(&self) -> Option<WindowId> {
    match self {
      Self::Id { id, .. }
      | Self::Title { id, .. }
      | Self::ProcessName { id, .. }
      | Self::Frame { id, .. }
      | Self::Position { id, .. }
      | Self::Size { id, .. }
      | Self::IsValid { id, .. }
      | Self::IsVisible { id, .. }
      | Self::IsMinimized { id, .. }
      | Self::IsMaximized { id, .. }
      | Self::IsResizable { id, .. }
      | Self::IsDesktopWindow { id, .. }
      | Self::SetFrame { id, .. }
      | Self::Resize { id, .. }
      | Self::Reposition { id, .. }
      | Self::Minimize { id }
      | Self::Maximize { id }
      | Self::Focus { id }
      | Self::Close { id } => Some(*id),
      #[cfg(target_os = "windows")]
      Self::ClassName { id, .. }
      | Self::FrameWithShadows { id, .. }
      | Self::ShadowBorders { id, .. }
      | Self::HasOwnerWindow { id, .. }
      | Self::HasWindowStyle { id, .. }
      | Self::HasWindowStyleEx { id, .. }
      | Self::SetWindowPos { id, .. }
      | Self::Show { id }
      | Self::Hide { id }
      | Self::Restore { id, .. }
      | Self::SetCloaked { id, .. }
      | Self::MarkFullscreen { id, .. }
      | Self::SetTaskbarVisibility { id, .. }
      | Self::AddWindowStyleEx { id, .. }
      | Self::SetZOrder { id, .. }
      | Self::SetTitleBarVisibility { id, .. }
      | Self::SetBorderColor { id, .. }
      | Self::SetCornerStyle { id, .. }
      | Self::SetTransparency { id, .. }
      | Self::AdjustTransparency { id, .. } => Some(*id),
      #[cfg(target_os = "macos")]
      Self::BundleId { id, .. }
      | Self::Role { id, .. }
      | Self::Subrole { id, .. }
      | Self::IsModal { id, .. }
      | Self::IsMain { id, .. } => Some(*id),
      Self::ResetFocus
      | Self::CursorPosition { .. }
      | Self::SetCursorPosition { .. } => None,
    }
  }

  /// Returns the [`PlatformMethod`] discriminator for this call.
  #[must_use]
  pub fn method(&self) -> PlatformMethod {
    match self {
      Self::Id { .. } => PlatformMethod::Id,
      Self::Title { .. } => PlatformMethod::Title,
      Self::ProcessName { .. } => PlatformMethod::ProcessName,
      Self::Frame { .. } => PlatformMethod::Frame,
      Self::Position { .. } => PlatformMethod::Position,
      Self::Size { .. } => PlatformMethod::Size,
      Self::IsValid { .. } => PlatformMethod::IsValid,
      Self::IsVisible { .. } => PlatformMethod::IsVisible,
      Self::IsMinimized { .. } => PlatformMethod::IsMinimized,
      Self::IsMaximized { .. } => PlatformMethod::IsMaximized,
      Self::IsResizable { .. } => PlatformMethod::IsResizable,
      Self::IsDesktopWindow { .. } => PlatformMethod::IsDesktopWindow,
      Self::SetFrame { .. } => PlatformMethod::SetFrame,
      Self::Resize { .. } => PlatformMethod::Resize,
      Self::Reposition { .. } => PlatformMethod::Reposition,
      Self::Minimize { .. } => PlatformMethod::Minimize,
      Self::Maximize { .. } => PlatformMethod::Maximize,
      Self::Focus { .. } => PlatformMethod::Focus,
      Self::Close { .. } => PlatformMethod::Close,
      #[cfg(target_os = "windows")]
      Self::ClassName { .. } => PlatformMethod::ClassName,
      #[cfg(target_os = "windows")]
      Self::FrameWithShadows { .. } => PlatformMethod::FrameWithShadows,
      #[cfg(target_os = "windows")]
      Self::ShadowBorders { .. } => PlatformMethod::ShadowBorders,
      #[cfg(target_os = "windows")]
      Self::HasOwnerWindow { .. } => PlatformMethod::HasOwnerWindow,
      #[cfg(target_os = "windows")]
      Self::HasWindowStyle { .. } => PlatformMethod::HasWindowStyle,
      #[cfg(target_os = "windows")]
      Self::HasWindowStyleEx { .. } => PlatformMethod::HasWindowStyleEx,
      #[cfg(target_os = "windows")]
      Self::SetWindowPos { .. } => PlatformMethod::SetWindowPos,
      #[cfg(target_os = "windows")]
      Self::Show { .. } => PlatformMethod::Show,
      #[cfg(target_os = "windows")]
      Self::Hide { .. } => PlatformMethod::Hide,
      #[cfg(target_os = "windows")]
      Self::Restore { .. } => PlatformMethod::Restore,
      #[cfg(target_os = "windows")]
      Self::SetCloaked { .. } => PlatformMethod::SetCloaked,
      #[cfg(target_os = "windows")]
      Self::MarkFullscreen { .. } => PlatformMethod::MarkFullscreen,
      #[cfg(target_os = "windows")]
      Self::SetTaskbarVisibility { .. } => {
        PlatformMethod::SetTaskbarVisibility
      }
      #[cfg(target_os = "windows")]
      Self::AddWindowStyleEx { .. } => PlatformMethod::AddWindowStyleEx,
      #[cfg(target_os = "windows")]
      Self::SetZOrder { .. } => PlatformMethod::SetZOrder,
      #[cfg(target_os = "windows")]
      Self::SetTitleBarVisibility { .. } => {
        PlatformMethod::SetTitleBarVisibility
      }
      #[cfg(target_os = "windows")]
      Self::SetBorderColor { .. } => PlatformMethod::SetBorderColor,
      #[cfg(target_os = "windows")]
      Self::SetCornerStyle { .. } => PlatformMethod::SetCornerStyle,
      #[cfg(target_os = "windows")]
      Self::SetTransparency { .. } => PlatformMethod::SetTransparency,
      #[cfg(target_os = "windows")]
      Self::AdjustTransparency { .. } => {
        PlatformMethod::AdjustTransparency
      }
      #[cfg(target_os = "macos")]
      Self::BundleId { .. } => PlatformMethod::BundleId,
      #[cfg(target_os = "macos")]
      Self::Role { .. } => PlatformMethod::Role,
      #[cfg(target_os = "macos")]
      Self::Subrole { .. } => PlatformMethod::Subrole,
      #[cfg(target_os = "macos")]
      Self::IsModal { .. } => PlatformMethod::IsModal,
      #[cfg(target_os = "macos")]
      Self::IsMain { .. } => PlatformMethod::IsMain,
      Self::ResetFocus => PlatformMethod::ResetFocus,
      Self::CursorPosition { .. } => PlatformMethod::CursorPosition,
      Self::SetCursorPosition { .. } => PlatformMethod::SetCursorPosition,
    }
  }
}

/// Records a sequence of [`PlatformCall`]s made on mock platform types.
///
/// Thread-safe via interior mutability. Shared between the test and mock
/// instances via `Arc<CallTracker>`. A single tracker can be shared across
/// multiple mock windows and the mock `Dispatcher`, allowing tests to
/// verify calls across all platform interactions.
///
/// # Why not `mockall`?
///
/// `mockall` requires `&mut self` access to set expectations and verify
/// call counts (via `checkpoint()` and `times()`). When a mock is wrapped
/// in `Arc<dyn Trait>` — as it must be for window management code that
/// stores windows as `Arc<dyn NativeWindow>` — there is no way to obtain
/// `&mut self` through the trait object. See
/// <https://github.com/asomers/mockall/issues/191> and
/// <https://github.com/asomers/mockall/issues/385>.
///
/// `CallTracker` solves this by using a side-channel: each mock pushes
/// calls to a shared `Arc<CallTracker>` via `&self`, so no mutable access
/// to the mock itself is needed. This allows call tracking on mocks that
/// have been erased behind `Arc<dyn NativeWindow>`.
#[derive(Debug, Default)]
pub struct CallTracker {
  calls: std::sync::Mutex<Vec<PlatformCall>>,
}

impl CallTracker {
  /// Records a platform call.
  ///
  /// # Panics
  ///
  /// Panics if the mutex is poisoned.
  pub fn push(&self, call: PlatformCall) {
    self.calls.lock().unwrap().push(call);
  }

  /// Returns a snapshot of all recorded calls.
  ///
  /// # Panics
  ///
  /// Panics if the mutex is poisoned.
  pub fn calls(&self) -> Vec<PlatformCall> {
    self.calls.lock().unwrap().clone()
  }

  /// Returns all calls made on the window with the given ID.
  ///
  /// Dispatcher calls (which have no window ID) are excluded.
  ///
  /// # Panics
  ///
  /// Panics if the mutex is poisoned.
  pub fn calls_for(&self, id: WindowId) -> Vec<PlatformCall> {
    self
      .calls
      .lock()
      .unwrap()
      .iter()
      .filter(|call| call.id() == Some(id))
      .cloned()
      .collect()
  }

  /// Returns all calls made on the `Dispatcher` (no window ID).
  ///
  /// # Panics
  ///
  /// Panics if the mutex is poisoned.
  pub fn dispatcher_calls(&self) -> Vec<PlatformCall> {
    self
      .calls
      .lock()
      .unwrap()
      .iter()
      .filter(|call| call.id().is_none())
      .cloned()
      .collect()
  }

  /// Returns the number of calls matching the given method.
  ///
  /// # Panics
  ///
  /// Panics if the mutex is poisoned.
  pub fn call_count(&self, method: PlatformMethod) -> usize {
    self
      .calls
      .lock()
      .unwrap()
      .iter()
      .filter(|call| call.method() == method)
      .count()
  }

  /// Returns the number of calls matching the given method for a specific
  /// window.
  ///
  /// # Panics
  ///
  /// Panics if the mutex is poisoned.
  pub fn call_count_for(
    &self,
    id: WindowId,
    method: PlatformMethod,
  ) -> usize {
    self
      .calls
      .lock()
      .unwrap()
      .iter()
      .filter(|call| call.id() == Some(id) && call.method() == method)
      .count()
  }

  /// Clears all recorded calls and returns them.
  ///
  /// # Panics
  ///
  /// Panics if the mutex is poisoned.
  pub fn take_calls(&self) -> Vec<PlatformCall> {
    std::mem::take(&mut self.calls.lock().unwrap())
  }
}

/// A mock `NativeWindow` with configurable return values and optional call
/// tracking.
///
/// Use the `bon`-generated builder to create instances:
///
/// ```
/// use std::sync::Arc;
/// use wm_platform::test_utils::{CallTracker, MockNativeWindow};
/// use wm_platform::WindowId;
///
/// let tracker = Arc::new(CallTracker::default());
/// let window = MockNativeWindow::mock()
///     .id(WindowId(1))
///     .title("my window".to_string())
///     .tracker(Arc::clone(&tracker))
///     .call();
/// ```
///
/// When `tracker` is `None`, method calls are not recorded. When `tracker`
/// is set, every method call pushes a [`PlatformCall`] variant to the
/// tracker.
///
/// # Why not `mockall`?
///
/// `mockall` requires `&mut self` access to set expectations and verify
/// call counts (via `checkpoint()` and `times()`). When a mock is wrapped
/// in `Arc<dyn Trait>` — as it must be for window management code that
/// stores windows as `Arc<dyn NativeWindow>` — there is no way to obtain
/// `&mut self` through the trait object. See
/// <https://github.com/asomers/mockall/issues/191> and
/// <https://github.com/asomers/mockall/issues/385>.
///
/// Instead, `MockNativeWindow` uses an optional [`CallTracker`]
/// side-channel: each method call pushes a [`PlatformCall`] via `&self`,
/// so no mutable access to the mock itself is needed. This allows call
/// tracking on mocks that have been erased behind `Arc<dyn NativeWindow>`.
#[allow(clippy::struct_excessive_bools)]
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
  tracker: Option<Arc<CallTracker>>,
}

#[bon]
impl MockNativeWindow {
  /// Creates a `MockNativeWindow` with configurable fields.
  ///
  /// All fields have defaults except `tracker`, which defaults to `None`
  /// (no call tracking). Pass `Some(Arc::clone(&tracker))` to enable
  /// call recording.
  #[builder]
  pub fn mock(
    #[builder(default = WindowId(0))] id: WindowId,
    #[builder(default = String::new())] title: String,
    #[builder(default = String::new())] process_name: String,
    #[builder(default = Rect::from_xy(0, 0, 100, 100))] frame: Rect,
    #[builder(default = (0.0_f64, 0.0_f64))] position: (f64, f64),
    #[builder(default = (100.0_f64, 100.0_f64))] size: (f64, f64),
    #[builder(default = true)] is_valid: bool,
    #[builder(default = true)] is_visible: bool,
    #[builder(default = false)] is_minimized: bool,
    #[builder(default = false)] is_maximized: bool,
    #[builder(default = true)] is_resizable: bool,
    #[builder(default = false)] is_desktop_window: bool,
    tracker: Option<Arc<CallTracker>>,
  ) -> Self {
    Self {
      id,
      title,
      process_name,
      frame,
      position,
      size,
      is_valid,
      is_visible,
      is_minimized,
      is_maximized,
      is_resizable,
      is_desktop_window,
      tracker,
    }
  }
}

impl NativeWindow for MockNativeWindow {
  fn id(&self) -> WindowId {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Id {
        id: self.id,
        result: self.id,
      });
    }
    self.id
  }

  fn title(&self) -> crate::Result<String> {
    let result = self.title.clone();
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Title {
        id: self.id,
        result: result.clone(),
      });
    }
    Ok(result)
  }

  fn process_name(&self) -> crate::Result<String> {
    let result = self.process_name.clone();
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::ProcessName {
        id: self.id,
        result: result.clone(),
      });
    }
    Ok(result)
  }

  fn frame(&self) -> crate::Result<Rect> {
    let result = self.frame.clone();
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Frame {
        id: self.id,
        result: result.clone(),
      });
    }
    Ok(result)
  }

  fn position(&self) -> crate::Result<(f64, f64)> {
    let result = self.position;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Position {
        id: self.id,
        result,
      });
    }
    Ok(result)
  }

  fn size(&self) -> crate::Result<(f64, f64)> {
    let result = self.size;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Size {
        id: self.id,
        result,
      });
    }
    Ok(result)
  }

  fn is_valid(&self) -> bool {
    let result = self.is_valid;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::IsValid {
        id: self.id,
        result,
      });
    }
    result
  }

  fn is_visible(&self) -> crate::Result<bool> {
    let result = self.is_visible;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::IsVisible {
        id: self.id,
        result,
      });
    }
    Ok(result)
  }

  fn is_minimized(&self) -> crate::Result<bool> {
    let result = self.is_minimized;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::IsMinimized {
        id: self.id,
        result,
      });
    }
    Ok(result)
  }

  fn is_maximized(&self) -> crate::Result<bool> {
    let result = self.is_maximized;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::IsMaximized {
        id: self.id,
        result,
      });
    }
    Ok(result)
  }

  fn is_resizable(&self) -> crate::Result<bool> {
    let result = self.is_resizable;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::IsResizable {
        id: self.id,
        result,
      });
    }
    Ok(result)
  }

  fn is_desktop_window(&self) -> crate::Result<bool> {
    let result = self.is_desktop_window;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::IsDesktopWindow {
        id: self.id,
        result,
      });
    }
    Ok(result)
  }

  fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::SetFrame {
        id: self.id,
        rect: rect.clone(),
      });
    }
    Ok(())
  }

  fn resize(&self, width: i32, height: i32) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Resize {
        id: self.id,
        width,
        height,
      });
    }
    Ok(())
  }

  fn reposition(&self, x: i32, y: i32) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Reposition { id: self.id, x, y });
    }
    Ok(())
  }

  fn minimize(&self) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Minimize { id: self.id });
    }
    Ok(())
  }

  fn maximize(&self) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Maximize { id: self.id });
    }
    Ok(())
  }

  fn focus(&self) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Focus { id: self.id });
    }
    Ok(())
  }

  fn close(&self) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Close { id: self.id });
    }
    Ok(())
  }

  #[cfg(target_os = "windows")]
  fn class_name(&self) -> crate::Result<String> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::ClassName {
        id: self.id,
        result: String::new(),
      });
    }
    Ok(String::new())
  }

  #[cfg(target_os = "windows")]
  fn shadow_borders(&self) -> crate::Result<RectDelta> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::ShadowBorders {
        id: self.id,
        result: RectDelta::zero(),
      });
    }
    Ok(RectDelta::zero())
  }

  #[cfg(target_os = "windows")]
  fn as_windows_ext(
    &self,
  ) -> crate::Result<&dyn crate::NativeWindowWindowsExt> {
    Ok(self)
  }

  #[cfg(target_os = "macos")]
  fn as_macos_ext(
    &self,
  ) -> crate::Result<&dyn crate::NativeWindowExtMacOs> {
    Ok(self)
  }
}

#[cfg(target_os = "windows")]
impl crate::NativeWindowWindowsExt for MockNativeWindow {
  fn from_handle(_handle: isize) -> crate::NativeWindowImpl {
    unimplemented!("MockNativeWindow::from_handle should not be called");
  }

  fn hwnd(&self) -> windows::Win32::Foundation::HWND {
    windows::Win32::Foundation::HWND(self.id.0)
  }

  fn class_name(&self) -> crate::Result<String> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::ClassName {
        id: self.id,
        result: String::new(),
      });
    }
    Ok(String::new())
  }

  fn frame_with_shadows(&self) -> crate::Result<Rect> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::FrameWithShadows {
        id: self.id,
        result: self.frame.clone(),
      });
    }
    Ok(self.frame.clone())
  }

  fn shadow_borders(&self) -> crate::Result<RectDelta> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::ShadowBorders {
        id: self.id,
        result: RectDelta::zero(),
      });
    }
    Ok(RectDelta::zero())
  }

  fn has_owner_window(&self) -> bool {
    let result = false;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::HasOwnerWindow {
        id: self.id,
        result,
      });
    }
    result
  }

  fn has_window_style(
    &self,
    style: windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE,
  ) -> bool {
    let result = true;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::HasWindowStyle {
        id: self.id,
        style,
        result,
      });
    }
    result
  }

  fn has_window_style_ex(&self, style: WINDOW_EX_STYLE) -> bool {
    let result = false;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::HasWindowStyleEx {
        id: self.id,
        style,
        result,
      });
    }
    result
  }

  fn set_window_pos(
    &self,
    z_order: &WindowZOrder,
    rect: &Rect,
    flags: SET_WINDOW_POS_FLAGS,
  ) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::SetWindowPos {
        id: self.id,
        z_order: z_order.clone(),
        rect: rect.clone(),
        flags,
      });
    }
    Ok(())
  }

  fn show(&self) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Show { id: self.id });
    }
    Ok(())
  }

  fn hide(&self) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Hide { id: self.id });
    }
    Ok(())
  }

  fn restore(&self, outer_frame: Option<&Rect>) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Restore {
        id: self.id,
        outer_frame: outer_frame.cloned(),
      });
    }
    Ok(())
  }

  fn set_cloaked(&self, cloaked: bool) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::SetCloaked {
        id: self.id,
        cloaked,
      });
    }
    Ok(())
  }

  fn mark_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::MarkFullscreen {
        id: self.id,
        fullscreen,
      });
    }
    Ok(())
  }

  fn set_taskbar_visibility(&self, visible: bool) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::SetTaskbarVisibility {
        id: self.id,
        visible,
      });
    }
    Ok(())
  }

  fn add_window_style_ex(&self, style: WINDOW_EX_STYLE) {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::AddWindowStyleEx { id: self.id, style });
    }
  }

  fn set_z_order(&self, zorder: &WindowZOrder) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::SetZOrder {
        id: self.id,
        zorder: zorder.clone(),
      });
    }
    Ok(())
  }

  fn set_title_bar_visibility(&self, visible: bool) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::SetTitleBarVisibility {
        id: self.id,
        visible,
      });
    }
    Ok(())
  }

  fn set_border_color(&self, color: Option<&Color>) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::SetBorderColor {
        id: self.id,
        color: color.cloned(),
      });
    }
    Ok(())
  }

  fn set_corner_style(
    &self,
    corner_style: &CornerStyle,
  ) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::SetCornerStyle {
        id: self.id,
        corner_style: corner_style.clone(),
      });
    }
    Ok(())
  }

  fn set_transparency(
    &self,
    opacity_value: &OpacityValue,
  ) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::SetTransparency {
        id: self.id,
        opacity_value: opacity_value.clone(),
      });
    }
    Ok(())
  }

  fn adjust_transparency(
    &self,
    opacity_delta: &Delta<OpacityValue>,
  ) -> crate::Result<()> {
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::AdjustTransparency {
        id: self.id,
        opacity_delta: opacity_delta.clone(),
      });
    }
    Ok(())
  }
}

#[cfg(target_os = "macos")]
impl crate::NativeWindowExtMacOs for MockNativeWindow {
  fn bundle_id(&self) -> Option<String> {
    let result = None;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::BundleId {
        id: self.id,
        result: result.clone(),
      });
    }
    result
  }

  fn role(&self) -> crate::Result<String> {
    let result = String::new();
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Role {
        id: self.id,
        result: result.clone(),
      });
    }
    Ok(result)
  }

  fn subrole(&self) -> crate::Result<String> {
    let result = String::new();
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::Subrole {
        id: self.id,
        result: result.clone(),
      });
    }
    Ok(result)
  }

  fn is_modal(&self) -> crate::Result<bool> {
    let result = false;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::IsModal {
        id: self.id,
        result,
      });
    }
    Ok(result)
  }

  fn is_main(&self) -> crate::Result<bool> {
    let result = false;
    if let Some(ref tracker) = self.tracker {
      tracker.push(PlatformCall::IsMain {
        id: self.id,
        result,
      });
    }
    Ok(result)
  }
}

impl std::fmt::Debug for MockNativeWindow {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MockNativeWindow")
      .field("id", &self.id)
      .field("title", &self.title)
      .field("process_name", &self.process_name)
      .field("frame", &self.frame)
      .field("position", &self.position)
      .field("size", &self.size)
      .field("is_valid", &self.is_valid)
      .field("is_visible", &self.is_visible)
      .field("is_minimized", &self.is_minimized)
      .field("is_maximized", &self.is_maximized)
      .field("is_resizable", &self.is_resizable)
      .field("is_desktop_window", &self.is_desktop_window)
      .field("tracker", &self.tracker.as_ref().map(|_| "CallTracker"))
      .finish()
  }
}

impl Dispatcher {
  /// Creates a mock `Dispatcher` for use in tests, without call tracking.
  ///
  /// Calling methods that require a real platform (e.g. `focused_window`,
  /// `displays`) will panic. Methods that the mock can safely stub
  /// (`reset_focus`, `cursor_position`, `set_cursor_position`) return
  /// sensible defaults.
  #[must_use]
  pub fn mock() -> Self {
    Self::new(None, Arc::new(AtomicBool::new(false)))
  }

  /// Creates a mock `Dispatcher` with call tracking.
  ///
  /// The mock stubs `reset_focus`, `cursor_position`, and
  /// `set_cursor_position` with linked behavior: `cursor_position`
  /// returns the current stored position (initially `(0, 0)`), and
  /// `set_cursor_position` updates it. All three methods record their
  /// calls in the shared [`CallTracker`].
  #[must_use]
  pub fn mock_with_tracker(tracker: Arc<CallTracker>) -> Self {
    let mut dispatcher = Self::new(None, Arc::new(AtomicBool::new(false)));
    dispatcher.tracker = Some(tracker);
    dispatcher
  }
}

impl NativeWindowImpl {
  /// Creates a mock `NativeWindow` for use in tests (no call tracking).
  ///
  /// The mock returns default values for all methods. For a mock with call
  /// tracking, use `MockNativeWindow::builder().tracker(...).call()`
  /// instead.
  #[must_use]
  pub fn mock() -> Arc<dyn NativeWindow> {
    Arc::new(MockNativeWindow::mock().call()) as Arc<dyn NativeWindow>
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
