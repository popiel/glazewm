//! Test utilities for building WM state for testing command and event
//! handlers.
//!
//! This module provides builders for constructing container tree
//! structures and `WmState` for testing purposes.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::tests::{simple_wm_state, TestWmStateBuilder};
//!
//! // Use fixture
//! #[test]
//! fn test_with_fixture() {
//!     let (test_state, config) = simple_wm_state();
//!     // ... test logic
//! }
//!
//! // Use builder for custom state
//! #[test]
//! fn test_with_builder() {
//!     let (state, config) = TestWmStateBuilder::new()
//!         .with_workspace("1")
//!         .with_tiling_window("Window 1")
//!         .build();
//! }
//! ```

use tokio::sync::mpsc;
use wm_common::{
  GapsConfig, ParsedConfig, TilingDirection, WorkspaceConfig,
};
use wm_platform::{Dispatcher, Rect};
use wm_state::TestWmState;

use crate::{
  commands::container::{attach_container, set_focused_descendant},
  models::{Container, Monitor, NativeMonitorProperties, Workspace},
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Re-exports for convenience
pub mod wm_state {
  use super::*;
  use crate::traits::WindowGetters;

  /// A minimal WM state wrapper for testing container tree structures.
  #[derive(Clone)]
  pub struct TestWmState {
    monitors: Vec<Monitor>,
  }

  impl TestWmState {
    pub fn new() -> Self {
      Self {
        monitors: Vec::new(),
      }
    }

    pub fn with_monitor(mut self, monitor: Monitor) -> Self {
      self.monitors.push(monitor);
      self
    }

    pub fn monitors(&self) -> &[Monitor] {
      &self.monitors
    }

    pub fn find_window_by_title(
      &self,
      title: &str,
    ) -> Option<crate::models::Container> {
      for monitor in &self.monitors {
        for workspace in monitor.workspaces() {
          for child in workspace.children() {
            if let Some(found) = Self::search_in_container(&child, title) {
              return Some(found);
            }
          }
        }
      }
      None
    }

    fn search_in_container(
      container: &crate::models::Container,
      title: &str,
    ) -> Option<crate::models::Container> {
      use crate::models::Container::*;

      match container {
        TilingWindow(w) => {
          if w.native_properties().title == title {
            Some(container.clone())
          } else {
            None
          }
        }
        NonTilingWindow(w) => {
          if w.native_properties().title == title {
            Some(container.clone())
          } else {
            None
          }
        }
        Split(split) => {
          for child in split.children() {
            if let Some(found) = Self::search_in_container(&child, title) {
              return Some(found);
            }
          }
          None
        }
        _ => None,
      }
    }

    pub fn count_tiling_windows(&self) -> usize {
      let mut count = 0;
      for monitor in &self.monitors {
        for workspace in monitor.workspaces() {
          count += Self::count_in_container(&workspace.as_container());
        }
      }
      count
    }

    fn count_in_container(container: &crate::models::Container) -> usize {
      use crate::models::Container::*;
      match container {
        TilingWindow(_) => 1,
        Split(split) => split
          .children()
          .into_iter()
          .map(|c| Self::count_in_container(&c))
          .sum(),
        Workspace(workspace) => workspace
          .children()
          .into_iter()
          .map(|c| Self::count_in_container(&c))
          .sum(),
        _ => 0,
      }
    }
  }

  impl Default for TestWmState {
    fn default() -> Self {
      Self::new()
    }
  }
}

/// Builder for creating full `WmState` instances for testing.
pub struct TestWmStateBuilder {
  monitors: Vec<Monitor>,
  workspaces: Vec<(String, Vec<WindowSpec>)>,
  focused_index: Option<(usize, usize)>,
}

/// Specification for a window to create.
#[derive(Clone)]
pub struct WindowSpec {
  pub title: String,
  pub is_floating: bool,
}

impl TestWmStateBuilder {
  pub fn new() -> Self {
    Self {
      monitors: Vec::new(),
      workspaces: Vec::new(),
      focused_index: None,
    }
  }

  pub fn with_monitor(mut self, name: &str) -> Self {
    let monitor = Monitor::new_test(create_test_monitor_props(name));
    self.monitors.push(monitor);
    self
  }

  pub fn with_workspace(mut self, name: &str) -> Self {
    self.workspaces.push((name.to_string(), Vec::new()));
    self
  }

  pub fn with_tiling_window(mut self, title: &str) -> Self {
    if let Some(workspace) = self.workspaces.last_mut() {
      workspace.1.push(WindowSpec {
        title: title.to_string(),
        is_floating: false,
      });
    }
    self
  }

  pub fn with_floating_window(mut self, title: &str) -> Self {
    if let Some(workspace) = self.workspaces.last_mut() {
      workspace.1.push(WindowSpec {
        title: title.to_string(),
        is_floating: true,
      });
    }
    self
  }

  #[allow(dead_code)]
  pub fn with_focused(
    mut self,
    workspace_idx: usize,
    window_idx: usize,
  ) -> Self {
    self.focused_index = Some((workspace_idx, window_idx));
    self
  }

  pub fn build(self) -> (WmState, UserConfig) {
    let (event_tx, _event_rx) = mpsc::unbounded_channel();
    let (exit_tx, _exit_rx) = mpsc::unbounded_channel();
    let dispatcher = Dispatcher::new_test();
    let mut state = WmState::new(dispatcher, event_tx, exit_tx);

    let mut config = default_test_config();

    // Add monitors from builder or create defaults
    if self.monitors.is_empty() {
      for monitor_name in ["DP-1", "DP-2"] {
        let monitor =
          Monitor::new_test(create_test_monitor_props(monitor_name));
        attach_container(
          &monitor.clone().into(),
          &state.root_container.clone().into(),
          None,
        )
        .unwrap();
      }
    } else {
      for monitor in &self.monitors {
        attach_container(
          &monitor.clone().into(),
          &state.root_container.clone().into(),
          None,
        )
        .unwrap();
      }
    }

    let monitors: Vec<Monitor> = state
      .root_container
      .children()
      .into_iter()
      .filter_map(|c| c.as_monitor().cloned())
      .collect();

    // Ensure at least one monitor exists
    assert!(!monitors.is_empty(), "No monitors available for test state");

    for (idx, (ws_name, windows)) in
      self.workspaces.into_iter().enumerate()
    {
      let monitor_idx = if monitors.is_empty() {
        0
      } else {
        idx.min(monitors.len() - 1)
      };
      let monitor = monitors
        .get(monitor_idx)
        .cloned()
        .unwrap_or_else(|| monitors[0].clone());

      let ws_config = WorkspaceConfig {
        name: ws_name.clone(),
        display_name: None,
        bind_to_monitor: None,
        keep_alive: false,
      };
      config.value.workspaces.push(ws_config.clone());

      let workspace =
        create_test_workspace_with_windows(&ws_name, &windows);

      attach_container(
        &workspace.clone().into(),
        &monitor.clone().into(),
        None,
      )
      .unwrap();

      if let Some((f_ws_idx, f_win_idx)) = self.focused_index {
        if idx == f_ws_idx {
          if let Some(win_spec) = windows.get(f_win_idx) {
            let focus_container = find_container_by_title(
              &workspace.clone().into(),
              &win_spec.title,
            )
            .unwrap_or_else(|| workspace.clone().into());
            set_focused_descendant(&focus_container, None);
          }
        }
      }

      state
        .pending_sync
        .queue_container_to_redraw(workspace.clone());
    }

    if self.focused_index.is_none() && !monitors.is_empty() {
      if let Some(monitor) = monitors.first() {
        if let Some(workspace) = monitor.displayed_workspace() {
          set_focused_descendant(&workspace.clone().into(), None);
        }
      }
    }

    (state, config)
  }
}

fn find_container_by_title(
  container: &Container,
  title: &str,
) -> Option<Container> {
  use crate::{models::Container::*, traits::WindowGetters};

  match container {
    TilingWindow(w) => {
      if w.native_properties().title == title {
        Some(container.clone())
      } else {
        None
      }
    }
    NonTilingWindow(w) => {
      if w.native_properties().title == title {
        Some(container.clone())
      } else {
        None
      }
    }
    Split(split) => {
      for child in split.children() {
        if let Some(found) = find_container_by_title(&child, title) {
          return Some(found);
        }
      }
      None
    }
    Workspace(ws) => {
      for child in ws.children() {
        if let Some(found) = find_container_by_title(&child, title) {
          return Some(found);
        }
      }
      None
    }
    _ => None,
  }
}

fn create_test_workspace_with_windows(
  name: &str,
  windows: &[WindowSpec],
) -> Workspace {
  use crate::models::{NonTilingWindow, SplitContainer, TilingWindow};

  let workspace = create_test_workspace(name);

  if windows.is_empty() {
    return workspace;
  }

  let has_tiling = windows.iter().any(|w| !w.is_floating);

  if has_tiling && windows.iter().any(|w| w.is_floating) {
    let split = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );

    for win_spec in windows.iter().filter(|w| !w.is_floating) {
      let window = TilingWindow::new_test(&win_spec.title);
      attach_container(&window.into(), &split.clone().into(), None)
        .unwrap();
    }

    attach_container(
      &split.clone().into(),
      &workspace.clone().into(),
      None,
    )
    .unwrap();

    for win_spec in windows.iter().filter(|w| w.is_floating) {
      let floating = NonTilingWindow::new_test_floating(&win_spec.title);
      attach_container(&floating.into(), &workspace.clone().into(), None)
        .unwrap();
    }
  } else if has_tiling {
    let split = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );

    for win_spec in windows {
      let window = TilingWindow::new_test(&win_spec.title);
      attach_container(&window.into(), &split.clone().into(), None)
        .unwrap();
    }

    attach_container(
      &split.clone().into(),
      &workspace.clone().into(),
      None,
    )
    .unwrap();
  } else {
    for win_spec in windows {
      let floating = NonTilingWindow::new_test_floating(&win_spec.title);
      attach_container(&floating.into(), &workspace.clone().into(), None)
        .unwrap();
    }
  }

  workspace
}

impl Default for TestWmStateBuilder {
  fn default() -> Self {
    Self::new()
  }
}

pub fn default_test_config() -> UserConfig {
  UserConfig::new_test(ParsedConfig::default())
}

/// Creates a simple WM state with one monitor and one workspace
/// with two tiling windows.
pub fn simple_wm_state() -> TestWmState {
  use crate::models::{SplitContainer, TilingWindow};

  let monitor = Monitor::new_test(create_test_monitor_props("DP-1"));
  let workspace = create_test_workspace("1");

  let split = SplitContainer::new(
    TilingDirection::Horizontal,
    GapsConfig::default(),
  );
  let window1 = TilingWindow::new_test("Window 1");
  let window2 = TilingWindow::new_test("Window 2");

  attach_container(&window1.into(), &split.clone().into(), None).unwrap();
  attach_container(&window2.into(), &split.clone().into(), None).unwrap();
  attach_container(&split.clone().into(), &workspace.clone().into(), None)
    .unwrap();
  attach_container(
    &workspace.clone().into(),
    &monitor.clone().into(),
    None,
  )
  .unwrap();

  TestWmState::new().with_monitor(monitor)
}

/// Creates a WM state with a nested H > V > H split structure.
pub fn nested_split_wm_state() -> TestWmState {
  let monitor = Monitor::new_test(create_test_monitor_props("DP-1"));
  let workspace = create_test_workspace_with_nested_split("1");

  attach_container(
    &workspace.clone().into(),
    &monitor.clone().into(),
    None,
  )
  .unwrap();

  TestWmState::new().with_monitor(monitor)
}

/// Creates a WM state with a split container containing 3+ children.
pub fn three_children_wm_state() -> TestWmState {
  let monitor = Monitor::new_test(create_test_monitor_props("DP-1"));
  let workspace = create_test_workspace_with_three_children("1");

  attach_container(
    &workspace.clone().into(),
    &monitor.clone().into(),
    None,
  )
  .unwrap();

  TestWmState::new().with_monitor(monitor)
}

/// Creates a WM state with mixed floating and tiling windows.
pub fn mixed_windows_wm_state() -> TestWmState {
  let monitor = Monitor::new_test(create_test_monitor_props("DP-1"));
  let workspace = create_test_workspace_with_mixed("1");

  attach_container(
    &workspace.clone().into(),
    &monitor.clone().into(),
    None,
  )
  .unwrap();

  TestWmState::new().with_monitor(monitor)
}

/// Creates a complex WM state with two monitors.
pub fn complex_wm_state() -> TestWmState {
  let monitor1 = Monitor::new_test(create_test_monitor_props("DP-1"));
  let ws1 = create_test_workspace_with_nested_split("1");
  let ws2 = create_test_workspace_with_three_children("2");

  attach_container(&ws1.clone().into(), &monitor1.clone().into(), None)
    .unwrap();
  attach_container(&ws2.clone().into(), &monitor1.clone().into(), None)
    .unwrap();

  let monitor2 = Monitor::new_test(create_test_monitor_props("DP-2"));
  let ws3 = create_test_workspace_with_mixed("3");

  attach_container(&ws3.clone().into(), &monitor2.clone().into(), None)
    .unwrap();

  TestWmState::new()
    .with_monitor(monitor1)
    .with_monitor(monitor2)
}

fn create_test_monitor_props(name: &str) -> NativeMonitorProperties {
  NativeMonitorProperties {
    #[cfg(target_os = "windows")]
    handle: 0isize,
    #[cfg(target_os = "windows")]
    hardware_id: Some(format!("HWID-{name}")),
    #[cfg(target_os = "windows")]
    device_path: Some(format!("\\\\?\\DISPLAY#{name}#0#{{GUID}}")),
    #[cfg(target_os = "macos")]
    device_uuid: format!("uuid-{name}"),
    device_name: name.to_string(),
    working_area: Rect::from_xy(0, 0, 1920, 1080),
    bounds: Rect::from_xy(0, 0, 1920, 1080),
    dpi: 96,
    scale_factor: 1.0,
  }
}

fn create_test_workspace(name: &str) -> Workspace {
  let config = WorkspaceConfig {
    name: name.to_string(),
    display_name: None,
    bind_to_monitor: None,
    keep_alive: false,
  };
  Workspace::new(
    config,
    GapsConfig::default(),
    TilingDirection::Horizontal,
  )
}

fn create_test_workspace_with_nested_split(name: &str) -> Workspace {
  use crate::models::{SplitContainer, TilingWindow};

  let workspace = create_test_workspace(name);

  let outer_h = SplitContainer::new(
    TilingDirection::Horizontal,
    GapsConfig::default(),
  );
  let v =
    SplitContainer::new(TilingDirection::Vertical, GapsConfig::default());

  let window1 = TilingWindow::new_test("Window 1");
  let window2 = TilingWindow::new_test("Window 2");
  let window3 = TilingWindow::new_test("Window 3");

  attach_container(&v.clone().into(), &outer_h.clone().into(), None)
    .unwrap();
  attach_container(&window1.clone().into(), &outer_h.clone().into(), None)
    .unwrap();
  attach_container(&window2.clone().into(), &v.clone().into(), None)
    .unwrap();
  attach_container(&window3.clone().into(), &v.clone().into(), None)
    .unwrap();
  attach_container(
    &outer_h.clone().into(),
    &workspace.clone().into(),
    None,
  )
  .unwrap();

  workspace
}

fn create_test_workspace_with_three_children(name: &str) -> Workspace {
  use crate::models::{SplitContainer, TilingWindow};

  let workspace = create_test_workspace(name);

  let split = SplitContainer::new(
    TilingDirection::Horizontal,
    GapsConfig::default(),
  );

  let window1 = TilingWindow::new_test("Window A");
  let window2 = TilingWindow::new_test("Window B");
  let window3 = TilingWindow::new_test("Window C");

  attach_container(&window1.clone().into(), &split.clone().into(), None)
    .unwrap();
  attach_container(&window2.clone().into(), &split.clone().into(), None)
    .unwrap();
  attach_container(&window3.clone().into(), &split.clone().into(), None)
    .unwrap();
  attach_container(&split.clone().into(), &workspace.clone().into(), None)
    .unwrap();

  workspace
}

fn create_test_workspace_with_mixed(name: &str) -> Workspace {
  use crate::models::{NonTilingWindow, SplitContainer, TilingWindow};

  let workspace = create_test_workspace(name);

  let split = SplitContainer::new(
    TilingDirection::Horizontal,
    GapsConfig::default(),
  );

  let tiling_window = TilingWindow::new_test("Tiling Window");
  let floating_window =
    NonTilingWindow::new_test_floating("Floating Window");

  attach_container(
    &tiling_window.clone().into(),
    &split.clone().into(),
    None,
  )
  .unwrap();
  attach_container(&split.clone().into(), &workspace.clone().into(), None)
    .unwrap();
  attach_container(
    &floating_window.clone().into(),
    &workspace.clone().into(),
    None,
  )
  .unwrap();

  workspace
}

/// Builder for creating custom test window configurations.
pub struct TestWindowBuilder {
  title: String,
  gaps_config: GapsConfig,
}

impl TestWindowBuilder {
  pub fn new(title: &str) -> Self {
    Self {
      title: title.to_string(),
      gaps_config: GapsConfig::default(),
    }
  }

  #[allow(dead_code)]
  pub fn with_gaps_config(mut self, gaps_config: GapsConfig) -> Self {
    self.gaps_config = gaps_config;
    self
  }

  pub fn build_tiling(self) -> crate::models::TilingWindow {
    crate::models::TilingWindow::new_test_titled(
      None,
      &self.title,
      self.gaps_config,
    )
  }

  #[allow(dead_code)]
  pub fn build_floating(self) -> crate::models::NonTilingWindow {
    crate::models::NonTilingWindow::new_test_floating(&self.title)
  }

  #[allow(dead_code)]
  pub fn build_minimized(self) -> crate::models::NonTilingWindow {
    crate::models::NonTilingWindow::new_test_minimized(&self.title)
  }
}

/// Builder for creating custom test split configurations.
pub struct TestSplitBuilder {
  tiling_direction: TilingDirection,
  gaps_config: GapsConfig,
  children: Vec<crate::models::Container>,
}

impl TestSplitBuilder {
  pub fn new() -> Self {
    Self {
      tiling_direction: TilingDirection::Horizontal,
      gaps_config: GapsConfig::default(),
      children: Vec::new(),
    }
  }

  pub fn with_tiling_direction(
    mut self,
    direction: TilingDirection,
  ) -> Self {
    self.tiling_direction = direction;
    self
  }

  #[allow(dead_code)]
  pub fn with_gaps_config(mut self, gaps_config: GapsConfig) -> Self {
    self.gaps_config = gaps_config;
    self
  }

  pub fn with_child(mut self, child: crate::models::Container) -> Self {
    self.children.push(child);
    self
  }

  pub fn build(self) -> crate::models::SplitContainer {
    use crate::models::SplitContainer;

    let split =
      SplitContainer::new(self.tiling_direction, self.gaps_config);

    for child in &self.children {
      attach_container(child, &split.clone().into(), None).unwrap();
    }

    split
  }
}

impl Default for TestSplitBuilder {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod test_helpers {
  use super::*;
  use crate::traits::{TilingDirectionGetters, WindowGetters};

  #[test]
  fn test_simple_wm_state() {
    let state = simple_wm_state();
    assert_eq!(state.monitors().len(), 1);
  }

  #[test]
  fn test_complex_wm_state() {
    let state = complex_wm_state();
    assert_eq!(state.monitors().len(), 2);
  }

  #[test]
  fn test_nested_split_has_windows() {
    let state = nested_split_wm_state();
    assert!(state.find_window_by_title("Window 1").is_some());
    assert!(state.find_window_by_title("Window 2").is_some());
    assert!(state.find_window_by_title("Window 3").is_some());
  }

  #[test]
  fn test_three_children_has_windows() {
    let state = three_children_wm_state();
    assert!(state.find_window_by_title("Window A").is_some());
    assert!(state.find_window_by_title("Window B").is_some());
    assert!(state.find_window_by_title("Window C").is_some());
  }

  #[test]
  fn test_mixed_windows_has_both_types() {
    let state = mixed_windows_wm_state();
    assert!(state.find_window_by_title("Tiling Window").is_some());
    assert!(state.find_window_by_title("Floating Window").is_some());
  }

  #[test]
  fn test_count_tiling_windows_nested_split() {
    let state = nested_split_wm_state();
    assert_eq!(state.count_tiling_windows(), 3);
  }

  #[test]
  fn test_count_tiling_windows_three_children() {
    let state = three_children_wm_state();
    assert_eq!(state.count_tiling_windows(), 3);
  }

  #[test]
  fn test_count_tiling_windows_mixed() {
    let state = mixed_windows_wm_state();
    assert_eq!(state.count_tiling_windows(), 1);
  }

  #[test]
  fn test_window_builder() {
    let window = TestWindowBuilder::new("Custom Window").build_tiling();
    assert_eq!(window.native_properties().title, "Custom Window");
  }

  #[test]
  fn test_split_builder() {
    use crate::models::TilingWindow;

    let window1 = TilingWindow::new_test("W1");
    let window2 = TilingWindow::new_test("W2");

    let split = TestSplitBuilder::new()
      .with_tiling_direction(TilingDirection::Vertical)
      .with_child(window1.as_container())
      .with_child(window2.as_container())
      .build();

    assert_eq!(split.children().len(), 2);
    assert_eq!(split.tiling_direction(), TilingDirection::Vertical);
  }

  #[test]
  fn test_wm_state_builder_single_workspace() {
    let (state, _config) = TestWmStateBuilder::new()
      .with_monitor("DP-1")
      .with_workspace("1")
      .with_tiling_window("Window 1")
      .with_tiling_window("Window 2")
      .build();

    assert_eq!(state.monitors().len(), 1);
    assert_eq!(state.workspaces().len(), 1);
    assert_eq!(state.windows().len(), 2);
  }

  #[test]
  fn test_wm_state_builder_multiple_monitors() {
    let (state, _config) = TestWmStateBuilder::new()
      .with_monitor("DP-1")
      .with_workspace("1")
      .with_tiling_window("W1")
      .with_workspace("2")
      .with_tiling_window("W2")
      .with_monitor("DP-2")
      .with_workspace("3")
      .with_tiling_window("W3")
      .build();

    assert_eq!(state.monitors().len(), 2);
    assert_eq!(state.workspaces().len(), 3);
  }

  #[test]
  fn test_wm_state_builder_with_floating() {
    let (state, _config) = TestWmStateBuilder::new()
      .with_monitor("DP-1")
      .with_workspace("1")
      .with_tiling_window("T1")
      .with_floating_window("F1")
      .build();

    assert_eq!(state.windows().len(), 2);
    let has_tiling = state.windows().iter().any(|w| {
      matches!(w, crate::models::WindowContainer::TilingWindow(_))
    });
    let has_floating = state.windows().iter().any(|w| matches!(w, crate::models::WindowContainer::NonTilingWindow(nw) if nw.state() == wm_common::WindowState::Floating(wm_common::FloatingStateConfig::default())));
    assert!(has_tiling && has_floating);
  }
}
