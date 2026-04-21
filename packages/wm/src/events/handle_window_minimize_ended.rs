use tracing::info;
use wm_common::{try_warn, WindowState};
use wm_platform::WindowId;

use crate::{
  commands::window::update_window_state, traits::WindowGetters,
  user_config::UserConfig, wm_state::WmState,
};

pub fn handle_window_minimize_ended(
  native_window_id: WindowId,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let native_window =
    state.root_container.get_native_window(&native_window_id);
  let native_window_ref = native_window.as_ref().map(|w| w.as_ref());
  let found_window = if let Some(nw) = native_window_ref {
    state.window_from_native(nw)
  } else {
    None
  };

  // Update the window's state to not be minimized.
  if let Some(window) = found_window {
    let is_minimized = try_warn!(window.native().is_minimized());

    window.update_native_properties(|properties| {
      properties.is_minimized = is_minimized;
    });

    if !is_minimized && window.state() == WindowState::Minimized {
      info!("Window minimize ended: {window}");

      let target_state = window
        .prev_state()
        .unwrap_or(WindowState::default_from_config(&config.value));

      update_window_state(window.clone(), target_state, state, config)?;
    }
  }

  Ok(())
}
