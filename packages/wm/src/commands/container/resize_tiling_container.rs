use crate::{
  models::TilingContainer,
  traits::{CommonGetters, TilingSizeGetters, MIN_TILING_SIZE},
};

pub fn resize_tiling_container(
  container_to_resize: &TilingContainer,
  target_size: f32,
) {
  let tiling_siblings =
    container_to_resize.tiling_siblings().collect::<Vec<_>>();

  // Ignore cases where the container is the only child.
  if tiling_siblings.is_empty() {
    container_to_resize.set_tiling_size(1.);
    return;
  }

  // Prevent the container from being smaller than the minimum size, and
  // larger than the space available from sibling containers.
  #[allow(clippy::cast_precision_loss)]
  let clamped_target_size = target_size.clamp(
    MIN_TILING_SIZE,
    1. - (tiling_siblings.len() as f32 * MIN_TILING_SIZE),
  );

  let size_delta = clamped_target_size - container_to_resize.tiling_size();
  container_to_resize.set_tiling_size(clamped_target_size);

  // Get available tiling size amongst siblings.
  let available_size =
    tiling_siblings.iter().fold(0.0, |sum, container| {
      sum + container.tiling_size() - MIN_TILING_SIZE
    });

  // Distribute the available tiling size amongst its siblings.
  for sibling in &tiling_siblings {
    // Get percentage of resize that affects this container. Siblings are
    // resized in proportion to their current size (i.e. larger containers
    // are shrunk more).
    let resize_factor =
      (sibling.tiling_size() - MIN_TILING_SIZE) / available_size;

    let size_delta = resize_factor * size_delta;

    sibling.set_tiling_size(sibling.tiling_size() - size_delta);
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{GapsConfig, TilingDirection};

  use super::*;
  use crate::{
    commands::container::attach_container,
    models::{SplitContainer, TilingWindow},
    tests::three_children_wm_state,
    traits::{CommonGetters, TilingSizeGetters},
  };

  fn setup_split_with_windows() -> (
    SplitContainer,
    crate::models::Container,
    crate::models::Container,
  ) {
    let split = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );

    let window1 = TilingWindow::new_test("W1");
    let window2 = TilingWindow::new_test("W2");

    attach_container(&window1.clone().into(), &split.clone().into(), None)
      .unwrap();
    attach_container(&window2.clone().into(), &split.clone().into(), None)
      .unwrap();

    split.set_tiling_size(1.0);
    window1.set_tiling_size(0.5);
    window2.set_tiling_size(0.5);

    (split, window1.into(), window2.into())
  }

  #[test]
  fn resize_tiling_container_increases_window_size() {
    let _state = three_children_wm_state();
    let (_, window1, window2) = setup_split_with_windows();

    let window1_tiling = window1.as_tiling_container().unwrap();
    let window2_tiling = window2.as_tiling_container().unwrap();

    let size_before = window1_tiling.tiling_size();
    resize_tiling_container(&window1_tiling, 0.7);

    assert!(window1_tiling.tiling_size() > size_before);
    assert!(window1_tiling.tiling_size() > 0.65);
    assert!(window1_tiling.tiling_size() < 0.75);

    assert!(window2_tiling.tiling_size() < 0.5);
    assert!(window2_tiling.tiling_size() > 0.25);
  }

  #[test]
  fn resize_tiling_container_decreases_window_size() {
    let (_, window1, _) = setup_split_with_windows();

    let window1_tiling = window1.as_tiling_container().unwrap();

    let size_before = window1_tiling.tiling_size();
    resize_tiling_container(&window1_tiling, 0.3);

    assert!(window1_tiling.tiling_size() < size_before);
    assert!(window1_tiling.tiling_size() > 0.25);
    assert!(window1_tiling.tiling_size() < 0.35);
  }

  #[test]
  fn resize_tiling_container_distributes_size_among_siblings() {
    use crate::models::TilingContainer;

    let split = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );

    let window1 = TilingWindow::new_test("W1");
    let window2 = TilingWindow::new_test("W2");
    let window3 = TilingWindow::new_test("W3");

    attach_container(&window1.clone().into(), &split.clone().into(), None)
      .unwrap();
    attach_container(&window2.clone().into(), &split.clone().into(), None)
      .unwrap();
    attach_container(&window3.clone().into(), &split.clone().into(), None)
      .unwrap();

    split.set_tiling_size(1.0);
    window1.set_tiling_size(0.33);
    window2.set_tiling_size(0.33);
    window3.set_tiling_size(0.34);

    let window1_tiling = TilingContainer::TilingWindow(window1.clone());
    let window2_tiling = window2.as_tiling_container().unwrap();
    let window3_tiling = window3.as_tiling_container().unwrap();

    resize_tiling_container(&window1_tiling, 0.5);

    assert!((window1.tiling_size() - 0.5).abs() < 0.01);

    let total_size = window1.tiling_size()
      + window2_tiling.tiling_size()
      + window3_tiling.tiling_size();
    assert!((total_size - 1.0).abs() < 0.01);
  }

  #[test]
  fn resize_tiling_container_clamps_to_minimum_size() {
    let (_, window1, window2) = setup_split_with_windows();

    let window1_tiling = window1.as_tiling_container().unwrap();
    let window2_tiling = window2.as_tiling_container().unwrap();

    resize_tiling_container(&window1_tiling, 0.99);

    assert!(window1_tiling.tiling_size() >= MIN_TILING_SIZE);
    assert!(window1_tiling.tiling_size() < 1.0);

    assert!(
      window2_tiling.tiling_size() > 0.0,
      "Sibling should still have some size"
    );
    let total =
      window1_tiling.tiling_size() + window2_tiling.tiling_size();
    assert!(
      (total - 1.0).abs() < 0.01,
      "Total should be ~1.0, got {total}"
    );
  }

  #[test]
  fn resize_tiling_container_sets_full_size_for_only_child() {
    let split = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );

    let window = TilingWindow::new_test("Only");

    attach_container(&window.clone().into(), &split.clone().into(), None)
      .unwrap();

    split.set_tiling_size(1.0);
    window.set_tiling_size(0.5);

    let window_tiling = window.as_tiling_container().unwrap();

    resize_tiling_container(&window_tiling, 0.5);

    assert!((window.tiling_size() - 1.0).abs() < f32::EPSILON);
  }

  #[test]
  fn resize_tiling_container_maintains_total_size_one() {
    let (_, window1, window2) = setup_split_with_windows();

    let window1_tiling = window1.as_tiling_container().unwrap();
    let window2_tiling = window2.as_tiling_container().unwrap();

    for target in [0.2, 0.3, 0.4, 0.6, 0.8] {
      resize_tiling_container(&window1_tiling, target);

      let total =
        window1_tiling.tiling_size() + window2_tiling.tiling_size();
      assert!(
        (total - 1.0).abs() < 0.01,
        "Total size {total} at target {target}"
      );
    }
  }
}
