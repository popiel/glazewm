use tracing::info;
use wm_common::{try_warn, WindowState};
use wm_platform::WindowId;

use crate::{
  commands::{
    container::set_focused_descendant, window::update_window_state,
  },
  traits::WindowGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_minimized(
  native_window_id: WindowId,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let native_window =
    state.root_container.get_native_window(native_window_id);
  let native_window_ref = native_window.as_ref().map(AsRef::as_ref);
  let found_window = if let Some(nw) = native_window_ref {
    state.window_from_native(nw)
  } else {
    None
  };

  // Update the window's state to be minimized.
  if let Some(window) = found_window {
    let is_minimized = try_warn!(window.native().is_minimized());

    window.update_native_properties(|properties| {
      properties.is_minimized = is_minimized;
    });

    if is_minimized && window.state() != WindowState::Minimized {
      info!("Window minimized: {window}");

      let window = update_window_state(
        window.clone(),
        WindowState::Minimized,
        state,
        config,
      )?;

      // Clear the drag state, as a window can be minimized while
      // being dragged (e.g. via `toggle-minimized`).
      // TODO: Investigate other code paths where the drag state should be
      // cleared (e.g. most commands that call `update_window_state`).
      window.set_active_drag(None);

      // Focus should be reassigned after a window has been minimized.
      if let Some(focus_target) = state.focus_target_after_removal(&window)
      {
        set_focused_descendant(&focus_target, None);
        state.pending_sync.queue_focus_change().queue_cursor_jump();
        state.unmanaged_or_minimized_timestamp =
          Some(std::time::Instant::now());
      }
    }
  }

  Ok(())
}
