use anyhow::Context;
use wm_common::try_warn;

use crate::{
  commands::monitor::{
    add_monitor, move_bounded_workspaces_to_new_monitor, remove_monitor,
    sort_monitors, update_monitor,
  },
  models::{Monitor, NativeMonitorProperties},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_display_settings_changed(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  tracing::info!("Display settings changed.");

  // Ignore the event if retrieval of the displays or their properties
  // fails (can happen transiently during sleep/wake).
  let displays = try_warn!(state
    .dispatcher
    .sorted_displays()
    .map_err(anyhow::Error::from)
    .and_then(|displays| {
      displays
        .into_iter()
        .map(|display| {
          let properties = NativeMonitorProperties::try_from(&display)?;
          Ok((display, properties))
        })
        .try_collect::<Vec<_>>()
    }));

  let mut pending_monitors = state.monitors();
  let mut unmatched_displays = Vec::new();

  // Match each display to an existing monitor and update it.
  for (display, properties) in displays {
    match find_matching_monitor(&pending_monitors, &properties) {
      Some((monitor, index)) => {
        update_monitor(monitor, &display, properties, state)?;
        pending_monitors.remove(index);
      }
      None => unmatched_displays.push((display, properties)),
    }
  }

  let mut new_monitors: Vec<Monitor> = Vec::new();

  // Pair unmatched displays with unmatched monitors, or add new ones.
  for (display, properties) in unmatched_displays {
    if pending_monitors.is_empty() {
      let monitor = add_monitor(display, properties, state)?;
      new_monitors.push(monitor);
    } else {
      let monitor = pending_monitors.remove(0);
      update_monitor(&monitor, &display, properties, state)?;
    }
  }

  // Remove monitors that no longer have a corresponding display and move
  // their workspaces to other monitors.
  //
  // Prevent removal of the last monitor (i.e. for when all monitors are
  // disconnected). This will cause the WM's monitors to temporarily
  // mismatch the OS monitor state, however, it'll be updated correctly
  // when a new monitor is connected again.
  for monitor in pending_monitors {
    if state.monitors().len() > 1 {
      remove_monitor(monitor, state, config)?;
    }
  }

  // Sort monitors by position.
  sort_monitors(&state.root_container)?;

  for new_monitor in new_monitors {
    move_bounded_workspaces_to_new_monitor(&new_monitor, state, config)?;
  }

  for window in state.windows() {
    // Display setting changes can spread windows out sporadically, so mark
    // all windows as needing a DPI adjustment (just in case).
    window.set_has_pending_dpi_adjustment(true);

    // Need to update floating position of moved windows when a monitor is
    // disconnected or if the primary display is changed. The primary
    // display dictates the position of 0,0.
    let workspace = window.workspace().context("No workspace.")?;

    let should_recenter = if window.has_custom_floating_placement() {
      let workspace_rect = workspace.to_rect()?;

      // Keep the placement if it still intersects the workspace, since
      // `PlatformEvent::DisplaySettingsChanged` can be triggered by
      // non-monitor changes (e.g. unplugging a USB device).
      window
        .floating_placement()
        .intersection_area(&workspace_rect)
        == 0
    } else {
      true
    };

    if should_recenter {
      window.set_floating_placement(
        window
          .floating_placement()
          .translate_to_center(&workspace.to_rect()?),
      );
    }
  }

  // Redraw full container tree.
  state
    .pending_sync
    .queue_container_to_redraw(state.root_container.clone());

  Ok(())
}

/// Finds the monitor matching the given display properties.
///
/// Returns the monitor and its index within the list of monitors.
fn find_matching_monitor<'a>(
  monitors: &'a [Monitor],
  properties: &NativeMonitorProperties,
) -> Option<(&'a Monitor, usize)> {
  monitors.iter().enumerate().find_map(|(index, monitor)| {
    let existing = monitor.native_properties();

    let is_match = {
      #[cfg(target_os = "macos")]
      {
        existing.device_uuid == properties.device_uuid
      }

      // On Windows, match the monitor by:
      // 1. Its handle
      // 2. Its device path
      // 3. Its hardware ID (if unique)
      //
      // Monitor handles and device paths are unique, but can change over
      // time. The hardware ID is not guaranteed to be unique, so we
      // match against that last.
      #[cfg(target_os = "windows")]
      {
        existing.handle == properties.handle
          || existing.device_path.as_deref().is_some_and(|device_path| {
            properties.device_path.as_deref() == Some(device_path)
          })
          || existing.hardware_id.as_deref().is_some_and(|hardware_id| {
            let is_unique = monitors
              .iter()
              .filter(|other_monitor| {
                other_monitor.native_properties().hardware_id.as_deref()
                  == Some(hardware_id)
              })
              .count()
              == 1;

            is_unique
              && properties.hardware_id.as_deref() == Some(hardware_id)
          })
      }
    };

    is_match.then_some((monitor, index))
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::Monitor;

  fn make_properties(
    handle: isize,
    device_path: Option<&str>,
    hardware_id: Option<&str>,
  ) -> NativeMonitorProperties {
    NativeMonitorProperties {
      #[cfg(target_os = "windows")]
      handle,
      #[cfg(target_os = "windows")]
      hardware_id: hardware_id.map(String::from),
      #[cfg(target_os = "windows")]
      device_path: device_path.map(String::from),
      #[cfg(target_os = "macos")]
      device_uuid: String::new(),
      device_name: String::new(),
      working_area: wm_platform::Rect::from_xy(0, 0, 1920, 1080),
      bounds: wm_platform::Rect::from_xy(0, 0, 1920, 1080),
      dpi: 96,
      scale_factor: 1.0,
    }
  }

  fn make_monitor(
    handle: isize,
    device_path: Option<&str>,
    hardware_id: Option<&str>,
  ) -> Monitor {
    Monitor::new_test(make_properties(handle, device_path, hardware_id))
  }

  #[test]
  fn find_matching_monitor_by_handle() {
    let monitors =
      [make_monitor(1, None, None), make_monitor(2, None, None)];
    let properties = make_properties(2, None, None);

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_some());
    let (_monitor, index) = result.unwrap();
    assert_eq!(index, 1);
  }

  #[test]
  fn find_matching_monitor_by_device_path() {
    let monitors = [
      make_monitor(1, Some("DP-1"), None),
      make_monitor(2, Some("HDMI-1"), None),
    ];
    let properties = make_properties(999, Some("HDMI-1"), None);

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_some());
    let (_monitor, index) = result.unwrap();
    assert_eq!(index, 1);
  }

  #[test]
  fn find_matching_monitor_by_unique_hardware_id() {
    let monitors = [
      make_monitor(1, None, Some("MONITOR1")),
      make_monitor(2, None, Some("MONITOR2")),
    ];
    let properties = make_properties(999, None, Some("MONITOR2"));

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_some());
    let (_monitor, index) = result.unwrap();
    assert_eq!(index, 1);
  }

  #[test]
  fn no_match_when_hardware_id_not_unique() {
    let monitors = [
      make_monitor(1, None, Some("SAME_ID")),
      make_monitor(2, None, Some("SAME_ID")),
    ];
    let properties = make_properties(999, None, Some("SAME_ID"));

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_none());
  }

  #[test]
  fn no_match_when_no_matching_properties() {
    let monitors = [
      make_monitor(1, Some("DP-1"), None),
      make_monitor(2, Some("HDMI-1"), None),
    ];
    let properties = make_properties(999, Some("DISPLAY_PORT_3"), None);

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_none());
  }

  #[test]
  fn no_match_empty_monitors_list() {
    let monitors: [Monitor; 0] = [];
    let properties = make_properties(1, None, None);

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_none());
  }

  #[test]
  fn handle_takes_precedence_over_device_path() {
    let monitors = [
      make_monitor(1, Some("DP-1"), Some("HW1")),
      make_monitor(2, Some("DP-2"), Some("HW2")),
    ];
    let properties = make_properties(1, Some("DP-2"), Some("HW2"));

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_some());
    let (monitor, index) = result.unwrap();
    assert_eq!(index, 0);
    assert_eq!(monitor.native_properties().handle, 1);
  }

  #[test]
  #[cfg(target_os = "windows")]
  fn handle_takes_precedence_over_hardware_id() {
    let monitors = [
      make_monitor(1, None, Some("HW1")),
      make_monitor(2, None, Some("HW2")),
    ];
    let properties = make_properties(1, None, Some("HW2"));

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_some());
    let (monitor, index) = result.unwrap();
    assert_eq!(index, 0);
    assert_eq!(monitor.native_properties().handle, 1);
  }

  #[test]
  #[cfg(target_os = "windows")]
  fn device_path_takes_precedence_over_hardware_id() {
    let monitors = [
      make_monitor(999, Some("DP-1"), Some("HW1")),
      make_monitor(998, Some("DP-2"), Some("HW2")),
    ];
    let properties = make_properties(1000, Some("DP-1"), Some("HW2"));

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_some());
    let (monitor, index) = result.unwrap();
    assert_eq!(index, 0);
    assert_eq!(
      monitor.native_properties().device_path,
      Some("DP-1".to_string())
    );
  }

  #[test]
  fn first_matching_monitor_is_returned() {
    let monitors = [
      make_monitor(1, Some("DP-1"), None),
      make_monitor(2, Some("DP-1"), None),
    ];
    let properties = make_properties(999, Some("DP-1"), None);

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_some());
    let (_monitor, index) = result.unwrap();
    assert_eq!(index, 0);
  }

  #[test]
  fn none_properties_matches_none_monitor_properties() {
    let monitors =
      [make_monitor(1, None, None), make_monitor(2, None, None)];
    let properties = make_properties(999, None, None);

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_none());
  }

  #[test]
  fn partially_matching_properties_do_not_match() {
    let monitors = [
      make_monitor(1, Some("DP-1"), Some("HW1")),
      make_monitor(2, Some("DP-2"), Some("HW2")),
    ];
    let properties = make_properties(999, Some("DP-1"), Some("HW2"));

    let result = find_matching_monitor(&monitors, &properties);
    assert!(result.is_some());
    let (_monitor, index) = result.unwrap();
    assert_eq!(index, 0);
  }
}
