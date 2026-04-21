use std::{
  collections::HashMap,
  time::{Duration, Instant},
};

use uuid::Uuid;
use wm_platform::WindowId;

use crate::{
  models::{Container, Workspace},
  traits::CommonGetters,
};

#[derive(Debug)]
struct DelayedBorderEffect {
  window_id: WindowId,
  due: Instant,
}

#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PendingSync {
  /// Containers (and their descendants) that have a pending redraw.
  containers_to_redraw: HashMap<Uuid, Container>,

  /// Workspaces where z-order should be updated. Windows that match the
  /// focused window's state should be brought to the front.
  workspaces_to_reorder: Vec<Workspace>,

  /// Whether native focus should be reassigned to the WM's focused
  /// container.
  needs_focus_update: bool,

  /// Whether window effect for the focused window should be updated.
  needs_focused_effect_update: bool,

  /// Whether window effects for all windows should be updated.
  needs_all_effects_update: bool,

  /// Whether to jump the cursor to the focused container (if enabled in
  /// user config).
  needs_cursor_jump: bool,

  /// Delayed border effects waiting to be applied after a delay.
  delayed_border_effects: Vec<DelayedBorderEffect>,
}

impl PendingSync {
  pub fn has_changes(&self) -> bool {
    !self.containers_to_redraw.is_empty()
      || !self.workspaces_to_reorder.is_empty()
      || self.needs_focus_update
      || self.needs_focused_effect_update
      || self.needs_all_effects_update
      || self.needs_cursor_jump
      || self.has_due_border_effects()
  }

  pub fn clear(&mut self) -> &mut Self {
    self.containers_to_redraw.clear();
    self.workspaces_to_reorder.clear();
    self.needs_focus_update = false;
    self.needs_focused_effect_update = false;
    self.needs_all_effects_update = false;
    self.needs_cursor_jump = false;
    self.delayed_border_effects.clear();
    self
  }

  pub fn queue_container_to_redraw<T>(&mut self, container: T) -> &mut Self
  where
    T: Into<Container>,
  {
    let container: Container = container.into();
    self.containers_to_redraw.insert(container.id(), container);
    self
  }

  pub fn queue_containers_to_redraw<I, T>(
    &mut self,
    containers: I,
  ) -> &mut Self
  where
    I: IntoIterator<Item = T>,
    T: Into<Container>,
  {
    for container in containers {
      let container: Container = container.into();
      self.containers_to_redraw.insert(container.id(), container);
    }

    self
  }

  pub fn dequeue_container_from_redraw<T>(
    &mut self,
    container: T,
  ) -> &mut Self
  where
    T: Into<Container>,
  {
    self.containers_to_redraw.remove(&container.into().id());
    self
  }

  pub fn queue_workspace_to_reorder(
    &mut self,
    workspace: Workspace,
  ) -> &mut Self {
    self.workspaces_to_reorder.push(workspace);
    self
  }

  pub fn queue_focus_change(&mut self) -> &mut Self {
    self.needs_focus_update = true;
    self
  }

  pub fn queue_focused_effect_update(&mut self) -> &mut Self {
    self.needs_focused_effect_update = true;
    self
  }

  pub fn queue_all_effects_update(&mut self) -> &mut Self {
    self.needs_all_effects_update = true;
    self
  }

  pub fn queue_cursor_jump(&mut self) -> &mut Self {
    self.needs_cursor_jump = true;
    self
  }

  pub fn needs_focus_update(&self) -> bool {
    self.needs_focus_update
  }

  pub fn needs_focused_effect_update(&self) -> bool {
    self.needs_focused_effect_update
  }

  pub fn needs_all_effects_update(&self) -> bool {
    self.needs_all_effects_update
  }

  pub fn needs_cursor_jump(&self) -> bool {
    self.needs_cursor_jump
  }

  pub fn containers_to_redraw(&self) -> &HashMap<Uuid, Container> {
    &self.containers_to_redraw
  }

  pub fn workspaces_to_reorder(&self) -> &Vec<Workspace> {
    &self.workspaces_to_reorder
  }

  pub fn queue_delayed_border_effect(
    &mut self,
    window_id: WindowId,
    delay: Duration,
  ) {
    let due = Instant::now() + delay;
    if let Some(existing) = self
      .delayed_border_effects
      .iter_mut()
      .find(|e| e.window_id == window_id)
    {
      existing.due = due;
    } else {
      self
        .delayed_border_effects
        .push(DelayedBorderEffect { window_id, due });
    }
  }

  pub fn get_due_border_effects(&mut self) -> Vec<WindowId> {
    let now = Instant::now();
    let due: Vec<WindowId> = self
      .delayed_border_effects
      .iter()
      .filter(|e| e.due <= now)
      .map(|e| e.window_id)
      .collect();
    self.delayed_border_effects.retain(|e| e.due > now);
    due
  }

  pub fn remove_delayed_border_effect(&mut self, window_id: &WindowId) {
    self
      .delayed_border_effects
      .retain(|e| e.window_id != *window_id);
  }

  pub fn has_due_border_effects(&self) -> bool {
    let now = Instant::now();
    self.delayed_border_effects.iter().any(|e| e.due <= now)
  }
}
