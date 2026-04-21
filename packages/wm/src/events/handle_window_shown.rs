use std::rc::Rc;

use tracing::info;
use wm_common::{DisplayState, HideMethod};
use wm_platform::{NativeWindow, WindowId};

use crate::{
  commands::window::manage_window, traits::WindowGetters,
  user_config::UserConfig, wm_state::WmState,
};

pub fn handle_window_shown(
  native_window_id: WindowId,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let native_window =
    state.root_container.get_native_window(&native_window_id);
  let native_window_ref = native_window.as_ref().map(|w| w.as_ref());
  let found_window = if let Some(nw) = native_window_ref {
    state.window_from_native(nw)
  } else {
    None
  };

  if let Some(window) = found_window {
    info!("Window shown: {window}");

    // Update display state if window is already managed.
    if config.value.general.hide_method != HideMethod::PlaceInCorner
      && window.display_state() == DisplayState::Showing
    {
      window.set_display_state(DisplayState::Shown);
    } else {
      state.pending_sync.queue_container_to_redraw(window);
    }
  } else if !state.ignored_windows.contains(&native_window_id) {
    // If the window is not managed and not explicitly ignored, we need to
    // get or create the native window from the ID.
    if let Some(native_rc) =
      state.root_container.get_native_window(&native_window_id)
    {
      manage_window(native_rc, None, state, config)?;
    }
  }

  Ok(())
}
