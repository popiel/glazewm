use tracing::info;
use wm_common::{try_warn, WindowRuleEvent};
use wm_platform::WindowId;

use crate::{
  commands::window::run_window_rules, traits::WindowGetters,
  user_config::UserConfig, wm_state::WmState,
};

pub fn handle_window_title_changed(
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
    info!("Window title changed: {window}");

    let title = try_warn!(window.native().title());

    window.update_native_properties(|properties| {
      properties.title = title;
    });

    // Run window rules for title change events.
    run_window_rules(
      window,
      &WindowRuleEvent::TitleChange,
      state,
      config,
    )?;
  }

  Ok(())
}
