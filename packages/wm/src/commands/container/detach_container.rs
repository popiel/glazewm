use anyhow::Context;

use super::flatten_split_container;
use crate::{
  models::Container,
  traits::{CommonGetters, TilingSizeGetters, MIN_TILING_SIZE},
};

/// Removes a container from the tree.
///
/// If the container is a tiling container, the siblings will be resized to
/// fill the freed up space. Will flatten empty parent split containers.
#[allow(clippy::needless_pass_by_value)]
pub fn detach_container(child_to_remove: Container) -> anyhow::Result<()> {
  // Flatten the parent split container if it'll be empty after removing
  // the child.
  if let Some(split_parent) = child_to_remove
    .parent()
    .and_then(|parent| parent.as_split().cloned())
  {
    if split_parent.child_count() == 1 {
      flatten_split_container(split_parent)?;
    }
  }

  let parent = child_to_remove.parent().context("No parent.")?;

  parent
    .borrow_children_mut()
    .retain(|c| c.id() != child_to_remove.id());

  parent
    .borrow_child_focus_order_mut()
    .retain(|id| *id != child_to_remove.id());

  *child_to_remove.borrow_parent_mut() = None;

  // Resize the siblings if it is a tiling container.
  if let Ok(child_to_remove) = child_to_remove.as_tiling_container() {
    let tiling_siblings = parent.tiling_children().collect::<Vec<_>>();

    // TODO: Share logic with `resize_tiling_container`.
    let available_size =
      tiling_siblings.iter().fold(0.0, |sum, container| {
        sum + container.tiling_size() - MIN_TILING_SIZE
      });

    // Adjust size of the siblings based on the freed up space.
    for sibling in &tiling_siblings {
      let resize_factor =
        (sibling.tiling_size() - MIN_TILING_SIZE) / available_size;

      let size_delta = resize_factor * child_to_remove.tiling_size();
      sibling.set_tiling_size(sibling.tiling_size() + size_delta);
    }
  }

  Ok(())
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

  #[test]
  fn detach_container_removes_from_parent() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let initial_child_count = split.child_count();
    let children = split.children();
    let child_to_remove = children.into_iter().next().unwrap();

    detach_container(child_to_remove.clone()).unwrap();

    assert_eq!(split.child_count(), initial_child_count - 1);
    assert!(child_to_remove.parent().is_none());
  }

  #[test]
  fn detach_container_removes_from_child_focus_order() {
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
    let child_to_remove = children.into_iter().next().unwrap();
    let child_id = child_to_remove.id();

    detach_container(child_to_remove.clone()).unwrap();

    let focus_order = split.borrow_child_focus_order().clone();
    assert!(!focus_order.into_iter().any(|id| id == child_id));
  }

  #[test]
  fn detach_container_resizes_siblings() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let children = split.children().into_iter().collect::<Vec<_>>();
    let child_to_remove = children[0].clone();
    let sibling = children[1].clone();

    let sibling_size_before =
      sibling.as_tiling_container().unwrap().tiling_size();

    detach_container(child_to_remove).unwrap();

    let sibling_size_after =
      sibling.as_tiling_container().unwrap().tiling_size();
    assert!(sibling_size_after > sibling_size_before);
  }

  #[test]
  fn detach_container_sets_detached_flag() {
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
    let child_to_remove = children.into_iter().next().unwrap();

    detach_container(child_to_remove.clone()).unwrap();

    assert!(child_to_remove.is_detached());
  }

  #[test]
  fn detach_container_flattens_empty_split() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let nested_split = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    let window = TilingWindow::new_test("Only Window");

    attach_container(
      &window.clone().into(),
      &nested_split.clone().into(),
      None,
    )
    .unwrap();
    attach_container(
      &nested_split.clone().into(),
      &split.clone().into(),
      None,
    )
    .unwrap();

    let initial_child_count = split.child_count();

    detach_container(window.clone().into()).unwrap();

    assert!(window.is_detached());
    assert_eq!(split.child_count(), initial_child_count - 1);

    let split_children_after: Vec<_> =
      split.children().into_iter().collect();
    assert!(!split_children_after
      .iter()
      .any(|c| c.id() == nested_split.id()));
  }
}
