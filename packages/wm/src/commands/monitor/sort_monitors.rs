use crate::{
  models::RootContainer,
  traits::{CommonGetters, PositionGetters},
};

/// Sorts the root container's monitors from left-to-right and
/// top-to-bottom.
pub fn sort_monitors(root: &RootContainer) -> anyhow::Result<()> {
  let monitors = root.monitors();

  // Create a tuple of monitors and their rects.
  let mut monitors_with_rect = monitors
    .into_iter()
    .map(|monitor| {
      let rect = monitor.to_rect()?.clone();
      anyhow::Ok((monitor, rect))
    })
    .try_collect::<Vec<_>>()?;

  // Sort monitors from left-to-right, top-to-bottom.
  monitors_with_rect.sort_by(|(_, rect_a), (_, rect_b)| {
    if rect_a.x() == rect_b.x() {
      rect_a.y().cmp(&rect_b.y())
    } else {
      rect_a.x().cmp(&rect_b.x())
    }
  });

  *root.borrow_children_mut() = monitors_with_rect
    .into_iter()
    .map(|(monitor, _)| monitor.into())
    .collect();

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_platform::Rect;

  use super::*;
  use crate::{commands::container::attach_container, models::Monitor};

  fn create_monitor_with_position(name: &str, x: i32, y: i32) -> Monitor {
    let props = crate::models::NativeMonitorProperties {
      #[cfg(target_os = "windows")]
      handle: 0,
      #[cfg(target_os = "windows")]
      hardware_id: Some(format!("HWID-{name}")),
      #[cfg(target_os = "windows")]
      device_path: Some(format!("\\\\?\\DISPLAY#{name}#0")),
      #[cfg(target_os = "macos")]
      device_uuid: format!("uuid-{}", name),
      device_name: name.to_string(),
      working_area: Rect::from_xy(x, y, 1920, 1080),
      bounds: Rect::from_xy(x, y, 1920, 1080),
      dpi: 96,
      scale_factor: 1.0,
    };
    Monitor::new_test(props)
  }

  #[test]
  fn sort_monitors_by_x_position() {
    let root = RootContainer::new();
    let monitor1 = create_monitor_with_position("M1", 1920, 0);
    let monitor2 = create_monitor_with_position("M2", 0, 0);
    let monitor3 = create_monitor_with_position("M3", 3840, 0);

    attach_container(&monitor1.clone().into(), &root.clone().into(), None)
      .unwrap();
    attach_container(&monitor2.clone().into(), &root.clone().into(), None)
      .unwrap();
    attach_container(&monitor3.clone().into(), &root.clone().into(), None)
      .unwrap();

    sort_monitors(&root).unwrap();

    let sorted_ids: Vec<_> = root
      .children()
      .iter()
      .map(|c| c.as_monitor().unwrap().native_properties().device_name)
      .collect();

    assert_eq!(sorted_ids, vec!["M2", "M1", "M3"]);
  }

  #[test]
  fn sort_monitors_by_y_when_same_x() {
    let root = RootContainer::new();
    let monitor1 = create_monitor_with_position("M1", 0, 1080);
    let monitor2 = create_monitor_with_position("M2", 0, 0);
    let monitor3 = create_monitor_with_position("M3", 0, 2160);

    attach_container(&monitor1.clone().into(), &root.clone().into(), None)
      .unwrap();
    attach_container(&monitor2.clone().into(), &root.clone().into(), None)
      .unwrap();
    attach_container(&monitor3.clone().into(), &root.clone().into(), None)
      .unwrap();

    sort_monitors(&root).unwrap();

    let sorted_ids: Vec<_> = root
      .children()
      .iter()
      .map(|c| c.as_monitor().unwrap().native_properties().device_name)
      .collect();

    assert_eq!(sorted_ids, vec!["M2", "M1", "M3"]);
  }

  #[test]
  fn sort_monitors_two_dimensional_grid() {
    let root = RootContainer::new();
    let monitor1 = create_monitor_with_position("M1", 1920, 0);
    let monitor2 = create_monitor_with_position("M2", 0, 1080);
    let monitor3 = create_monitor_with_position("M3", 0, 0);
    let monitor4 = create_monitor_with_position("M4", 1920, 1080);

    attach_container(&monitor1.clone().into(), &root.clone().into(), None)
      .unwrap();
    attach_container(&monitor2.clone().into(), &root.clone().into(), None)
      .unwrap();
    attach_container(&monitor3.clone().into(), &root.clone().into(), None)
      .unwrap();
    attach_container(&monitor4.clone().into(), &root.clone().into(), None)
      .unwrap();

    sort_monitors(&root).unwrap();

    let sorted_ids: Vec<_> = root
      .children()
      .iter()
      .map(|c| c.as_monitor().unwrap().native_properties().device_name)
      .collect();

    // Sort order: first by x (left-to-right), then by y (top-to-bottom)
    // M3: (0, 0), M2: (0, 1080), M1: (1920, 0), M4: (1920, 1080)
    assert_eq!(sorted_ids, vec!["M3", "M2", "M1", "M4"]);
  }

  #[test]
  fn sort_monitors_single_monitor() {
    let root = RootContainer::new();
    let monitor = create_monitor_with_position("M1", 100, 200);

    attach_container(&monitor.clone().into(), &root.clone().into(), None)
      .unwrap();

    sort_monitors(&root).unwrap();

    let sorted_ids: Vec<_> = root
      .children()
      .iter()
      .map(|c| c.as_monitor().unwrap().native_properties().device_name)
      .collect();

    assert_eq!(sorted_ids, vec!["M1"]);
  }

  #[test]
  fn sort_monitors_empty_root() {
    let root = RootContainer::new();
    sort_monitors(&root).unwrap();
    assert_eq!(root.children().len(), 0);
  }
}
