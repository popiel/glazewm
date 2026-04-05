use std::collections::VecDeque;

use anyhow::Context;

use crate::{
  models::SplitContainer,
  traits::{CommonGetters, TilingSizeGetters},
};

/// Removes a split container from the tree and moves its children
/// into the parent container.
///
/// The children will be resized to fit the size of the split container.
#[allow(clippy::needless_pass_by_value)]
pub fn flatten_split_container(
  split_container: SplitContainer,
) -> anyhow::Result<()> {
  let parent = split_container.parent().context("No parent.")?;

  let updated_children =
    split_container.children().into_iter().inspect(|child| {
      *child.borrow_parent_mut() = Some(parent.clone());

      // Resize tiling children to fit the size of the split container.
      if let Ok(tiling_child) = child.as_tiling_container() {
        tiling_child.set_tiling_size(
          split_container.tiling_size() * tiling_child.tiling_size(),
        );
      }
    });

  let index = split_container.index();
  let focus_index = split_container.focus_index();

  // Insert child at its original index in the parent.
  for (child_index, child) in updated_children.enumerate() {
    parent
      .borrow_children_mut()
      .insert(index + child_index, child);
  }

  // Insert child at its original focus index in the parent.
  for (child_focus_index, child_id) in split_container
    .borrow_child_focus_order()
    .iter()
    .enumerate()
  {
    parent
      .borrow_child_focus_order_mut()
      .insert(focus_index + child_focus_index, *child_id);
  }

  // Remove the split container from the tree.
  parent
    .borrow_children_mut()
    .retain(|c| c.id() != split_container.id());

  parent
    .borrow_child_focus_order_mut()
    .retain(|id| *id != split_container.id());

  *split_container.borrow_parent_mut() = None;
  *split_container.borrow_children_mut() = VecDeque::new();

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
  fn flatten_split_container_moves_children_to_grandparent() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let outer = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );
    let inner = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    let window1 = TilingWindow::new_test("W1");
    let window2 = TilingWindow::new_test("W2");

    outer.set_tiling_size(1.0);
    inner.set_tiling_size(0.5);
    window1.set_tiling_size(0.5);
    window2.set_tiling_size(0.5);

    attach_container(&window1.clone().into(), &inner.clone().into(), None)
      .unwrap();
    attach_container(&window2.clone().into(), &inner.clone().into(), None)
      .unwrap();
    attach_container(&inner.clone().into(), &outer.clone().into(), None)
      .unwrap();
    attach_container(
      &outer.clone().into(),
      &workspace.clone().into(),
      None,
    )
    .unwrap();

    assert_eq!(outer.child_count(), 1);

    flatten_split_container(inner.clone()).unwrap();

    assert_eq!(outer.child_count(), 2);

    let outer_children: Vec<_> = outer.children().into_iter().collect();
    assert!(outer_children.iter().any(|c| c
      .as_tiling_window()
      .is_some_and(|w| w.id() == window1.id())));
    assert!(outer_children.iter().any(|c| c
      .as_tiling_window()
      .is_some_and(|w| w.id() == window2.id())));
  }

  #[test]
  fn flatten_split_container_removes_split_from_parent() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let outer = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );
    let inner = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    let window = TilingWindow::new_test("W");

    outer.set_tiling_size(1.0);
    inner.set_tiling_size(0.5);
    window.set_tiling_size(1.0);

    attach_container(&window.clone().into(), &inner.clone().into(), None)
      .unwrap();
    attach_container(&inner.clone().into(), &outer.clone().into(), None)
      .unwrap();
    attach_container(
      &outer.clone().into(),
      &workspace.clone().into(),
      None,
    )
    .unwrap();

    let inner_id = inner.id();

    flatten_split_container(inner).unwrap();

    let outer_children: Vec<_> = outer.children().into_iter().collect();
    assert!(!outer_children.iter().any(|c| c.id() == inner_id));
  }

  #[test]
  fn flatten_split_container_updates_parent_pointers() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let outer = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );
    let inner = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    let window = TilingWindow::new_test("W");

    outer.set_tiling_size(1.0);
    inner.set_tiling_size(0.5);
    window.set_tiling_size(1.0);

    attach_container(&window.clone().into(), &inner.clone().into(), None)
      .unwrap();
    attach_container(&inner.clone().into(), &outer.clone().into(), None)
      .unwrap();
    attach_container(
      &outer.clone().into(),
      &workspace.clone().into(),
      None,
    )
    .unwrap();

    flatten_split_container(inner).unwrap();

    let window_parent =
      window.parent().expect("Window should have parent");
    assert_eq!(window_parent.id(), outer.id());
  }

  #[test]
  fn flatten_split_container_scales_tiling_sizes() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let outer = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );
    let inner = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    let window1 = TilingWindow::new_test("W1");
    let window2 = TilingWindow::new_test("W2");

    attach_container(&window1.clone().into(), &inner.clone().into(), None)
      .unwrap();
    attach_container(&window2.clone().into(), &inner.clone().into(), None)
      .unwrap();
    attach_container(&inner.clone().into(), &outer.clone().into(), None)
      .unwrap();
    attach_container(
      &outer.clone().into(),
      &workspace.clone().into(),
      None,
    )
    .unwrap();

    // Set tiling sizes after attaching
    outer.set_tiling_size(1.0);
    inner.set_tiling_size(0.6);
    window1.set_tiling_size(0.3);
    window2.set_tiling_size(0.7);

    flatten_split_container(inner).unwrap();

    let window1_size = window1.tiling_size();
    let window2_size = window2.tiling_size();

    let expected_w1 = 0.6 * 0.3;
    let expected_w2 = 0.6 * 0.7;

    assert!(
      (window1_size - expected_w1).abs() < 0.001,
      "Expected {expected_w1}, got {window1_size}"
    );
    assert!(
      (window2_size - expected_w2).abs() < 0.001,
      "Expected {expected_w2}, got {window2_size}"
    );
  }

  #[test]
  fn flatten_split_container_clears_split_children() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let outer = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );
    let inner = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    let window = TilingWindow::new_test("W");

    outer.set_tiling_size(1.0);
    inner.set_tiling_size(0.5);
    window.set_tiling_size(1.0);

    attach_container(&window.clone().into(), &inner.clone().into(), None)
      .unwrap();
    attach_container(&inner.clone().into(), &outer.clone().into(), None)
      .unwrap();
    attach_container(
      &outer.clone().into(),
      &workspace.clone().into(),
      None,
    )
    .unwrap();

    flatten_split_container(inner.clone()).unwrap();

    assert_eq!(inner.borrow_children_mut().len(), 0);
    assert!(inner.parent().is_none());
  }

  #[test]
  fn flatten_split_container_preserves_focus_order() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let outer = SplitContainer::new(
      TilingDirection::Horizontal,
      GapsConfig::default(),
    );
    let inner = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    let window1 = TilingWindow::new_test("W1");
    let window2 = TilingWindow::new_test("W2");

    outer.set_tiling_size(1.0);
    inner.set_tiling_size(0.5);
    window1.set_tiling_size(0.5);
    window2.set_tiling_size(0.5);

    attach_container(&window2.clone().into(), &inner.clone().into(), None)
      .unwrap();
    attach_container(&window1.clone().into(), &inner.clone().into(), None)
      .unwrap();
    attach_container(&inner.clone().into(), &outer.clone().into(), None)
      .unwrap();
    attach_container(
      &outer.clone().into(),
      &workspace.clone().into(),
      None,
    )
    .unwrap();

    flatten_split_container(inner).unwrap();

    let focus_order = outer.borrow_child_focus_order().clone();
    assert_eq!(focus_order.len(), 2);
    assert!(focus_order.contains(&window2.id()));
    assert!(focus_order.contains(&window1.id()));
  }
}
