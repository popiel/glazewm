use anyhow::bail;

use super::resize_tiling_container;
use crate::{
  models::Container,
  traits::{CommonGetters, TilingSizeGetters},
};

/// Inserts a child container at the specified index.
///
/// The inserted child will be resized to fit the available space.
pub fn attach_container(
  child: &Container,
  target_parent: &Container,
  target_index: Option<usize>,
) -> anyhow::Result<()> {
  if !child.is_detached() {
    bail!("Cannot attach an already attached container.");
  }

  if let Some(target_index) = target_index {
    // Ensure target index is within the bounds of the parent's children.
    let target_index = target_index.clamp(0, target_parent.child_count());

    // Insert the child at the specified index.
    target_parent
      .borrow_children_mut()
      .insert(target_index, child.clone());
  } else {
    target_parent.borrow_children_mut().push_back(child.clone());
  }

  target_parent
    .borrow_child_focus_order_mut()
    .push_back(child.id());

  *child.borrow_parent_mut() = Some(target_parent.clone());

  // Resize the child and its siblings if it is a tiling container.
  if let Ok(child) = child.as_tiling_container() {
    let tiling_siblings = child.tiling_siblings().collect::<Vec<_>>();

    if tiling_siblings.is_empty() {
      child.set_tiling_size(1.0);
      return Ok(());
    }

    // Set initial tiling size to 0, and then size up the container
    // to the target size.
    #[allow(clippy::cast_precision_loss)]
    let target_size = 1.0 / (tiling_siblings.len() + 1) as f32;
    child.set_tiling_size(0.0);
    resize_tiling_container(&child, target_size);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::{GapsConfig, TilingDirection};

  use super::*;
  use crate::{
    models::{SplitContainer, TilingWindow},
    tests::three_children_wm_state,
    traits::{CommonGetters, TilingSizeGetters, WindowGetters},
  };

  fn create_detached_window(title: &str) -> Container {
    let window =
      TilingWindow::new_test_titled(None, title, GapsConfig::default());
    window.into()
  }

  #[test]
  fn attach_container_adds_to_end_when_index_none() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let new_window = create_detached_window("New Window");
    let initial_child_count = split.child_count();

    attach_container(&new_window, &split.clone().into(), None).unwrap();

    assert_eq!(split.child_count(), initial_child_count + 1);
    assert!(new_window.parent().is_some());
  }

  #[test]
  fn attach_container_inserts_at_specific_index() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let new_window = create_detached_window("New Window");

    attach_container(&new_window, &split.clone().into(), Some(0)).unwrap();

    let children = split.children();
    let first_child = children.into_iter().next().unwrap();
    let window = first_child.as_tiling_window().unwrap();
    assert_eq!(window.native_properties().title, "New Window");
  }

  #[test]
  fn attach_container_sets_parent_reference() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let new_window = create_detached_window("New Window");

    attach_container(&new_window, &split.clone().into(), None).unwrap();

    let parent = new_window.parent().expect("Should have parent");
    assert_eq!(parent.id(), split.id());
  }

  #[test]
  fn attach_container_adds_to_child_focus_order() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let new_window = create_detached_window("New Window");

    attach_container(&new_window, &split.clone().into(), None).unwrap();

    let focus_order = split.borrow_child_focus_order().clone();
    assert!(focus_order.into_iter().any(|id| id == new_window.id()));
  }

  #[test]
  fn attach_container_resizes_tiling_window() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let new_window = create_detached_window("New Window");

    attach_container(&new_window, &split.clone().into(), None).unwrap();

    let tiling_window = new_window.as_tiling_window().unwrap();
    assert!(tiling_window.tiling_size() > 0.0);
    assert!(tiling_window.tiling_size() < 1.0);
  }

  #[test]
  fn attach_container_fails_if_already_attached() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let children = split.children();
    let attached_window = children.into_iter().next().unwrap();

    let result =
      attach_container(&attached_window, &split.clone().into(), None);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already attached"));
  }

  #[test]
  fn attach_container_to_empty_split_sets_full_size() {
    let empty_split = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );

    let new_window = create_detached_window("New Window");

    attach_container(&new_window, &empty_split.clone().into(), None)
      .unwrap();

    let tiling_window = new_window.as_tiling_window().unwrap();
    assert!((tiling_window.tiling_size() - 1.0).abs() < f32::EPSILON);
  }
}
