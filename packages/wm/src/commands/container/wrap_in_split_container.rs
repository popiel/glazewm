use std::collections::VecDeque;

use anyhow::Context;

use crate::{
  models::{Container, SplitContainer, TilingContainer},
  traits::{CommonGetters, TilingSizeGetters},
};

pub fn wrap_in_split_container(
  split_container: &SplitContainer,
  target_parent: &Container,
  target_children: &[TilingContainer],
) -> anyhow::Result<()> {
  let starting_index = target_children
    .iter()
    .map(CommonGetters::index)
    .min()
    .context("Failed to get starting index.")?;

  target_parent
    .borrow_children_mut()
    .insert(starting_index, split_container.clone().into());

  let starting_focus_index = target_children
    .iter()
    .map(CommonGetters::focus_index)
    .min()
    .context("Failed to get starting focus index.")?;

  target_parent
    .borrow_child_focus_order_mut()
    .insert(starting_focus_index, split_container.id());

  // Get the total tiling size amongst all children.
  let total_tiling_size = target_children
    .iter()
    .map(TilingSizeGetters::tiling_size)
    .sum::<f32>();

  let target_children_ids = target_children
    .iter()
    .map(CommonGetters::id)
    .collect::<Vec<_>>();

  let sorted_focus_ids = target_parent
    .borrow_child_focus_order()
    .iter()
    .filter(|id| target_children_ids.contains(id))
    .copied()
    .collect::<VecDeque<_>>();

  // Set the split container's parent and tiling size.
  *split_container.borrow_parent_mut() = Some(target_parent.clone());
  split_container.set_tiling_size(total_tiling_size);

  // Move the children from their original parent to the split container.
  for target_child in target_children {
    *target_child.borrow_parent_mut() =
      Some(split_container.clone().into());

    split_container
      .borrow_children_mut()
      .push_back(target_child.clone().into());

    target_parent
      .borrow_children_mut()
      .retain(|child| child != &target_child.clone().into());

    target_parent
      .borrow_child_focus_order_mut()
      .retain(|id| id != &target_child.id());

    // Scale the tiling size to the new split container.
    target_child
      .set_tiling_size(target_child.tiling_size() / total_tiling_size);
  }

  // Add original focus order to split container.
  *split_container.borrow_child_focus_order_mut() = sorted_focus_ids;

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::{GapsConfig, TilingDirection};

  use super::*;
  use crate::{
    models::SplitContainer,
    tests::three_children_wm_state,
    traits::{CommonGetters, TilingSizeGetters},
  };

  #[test]
  fn wrap_in_split_container_moves_children_to_split() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let children: Vec<_> = split.children().into_iter().collect();
    let window1 = children[0].as_tiling_container().unwrap();
    let window2 = children[1].as_tiling_container().unwrap();

    let new_split = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    wrap_in_split_container(
      &new_split,
      &split.clone().into(),
      &[window1.clone(), window2.clone()],
    )
    .unwrap();

    assert_eq!(new_split.child_count(), 2);

    let new_split_children: Vec<_> =
      new_split.children().into_iter().collect();
    assert!(new_split_children.iter().any(|c| c.id() == window1.id()));
    assert!(new_split_children.iter().any(|c| c.id() == window2.id()));
  }

  #[test]
  fn wrap_in_split_container_sets_parent_references() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let children: Vec<_> = split.children().into_iter().collect();
    let window1 = children[0].clone();
    let window2 = children[1].clone();

    let new_split = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    wrap_in_split_container(
      &new_split,
      &split.clone().into(),
      &[
        window1.as_tiling_container().unwrap(),
        window2.as_tiling_container().unwrap(),
      ],
    )
    .unwrap();

    assert_eq!(window1.parent().map(|p| p.id()), Some(new_split.id()));
    assert_eq!(window2.parent().map(|p| p.id()), Some(new_split.id()));
    assert_eq!(new_split.parent().map(|p| p.id()), Some(split.id()));
  }

  #[test]
  fn wrap_in_split_container_scales_tiling_sizes() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let children: Vec<_> = split.children().into_iter().collect();
    let window1_container = children[0].as_tiling_container().unwrap();
    let window2_container = children[1].as_tiling_container().unwrap();

    let window1_size_before = window1_container.tiling_size();
    let window2_size_before = window2_container.tiling_size();
    let total_size = window1_size_before + window2_size_before;

    let new_split = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    wrap_in_split_container(
      &new_split,
      &split.clone().into(),
      &[window1_container.clone(), window2_container.clone()],
    )
    .unwrap();

    let expected_w1 = window1_size_before / total_size;
    let expected_w2 = window2_size_before / total_size;

    assert!(
      (window1_container.tiling_size() - expected_w1).abs() < 0.001,
      "Expected {}, got {}",
      expected_w1,
      window1_container.tiling_size()
    );
    assert!(
      (window2_container.tiling_size() - expected_w2).abs() < 0.001,
      "Expected {}, got {}",
      expected_w2,
      window2_container.tiling_size()
    );

    assert!((new_split.tiling_size() - total_size).abs() < 0.001);
  }

  #[test]
  fn wrap_in_split_container_preserves_focus_order() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let children: Vec<_> = split.children().into_iter().collect();
    let window1 = children[0].clone();
    let window2 = children[1].clone();
    let window3 = children[2].clone();

    let new_split = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    wrap_in_split_container(
      &new_split,
      &split.clone().into(),
      &[
        window1.as_tiling_container().unwrap(),
        window2.as_tiling_container().unwrap(),
      ],
    )
    .unwrap();

    let focus_order = new_split.borrow_child_focus_order().clone();
    assert_eq!(focus_order.len(), 2);

    let parent_focus_order = split.borrow_child_focus_order().clone();
    assert!(parent_focus_order.contains(&new_split.id()));
    assert!(!parent_focus_order.contains(&window1.id()));
    assert!(!parent_focus_order.contains(&window2.id()));
    assert!(parent_focus_order.contains(&window3.id()));
  }

  #[test]
  fn wrap_in_split_container_removes_children_from_parent() {
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

    let children: Vec<_> = split.children().into_iter().collect();
    let window1 = children[0].clone();
    let window2 = children[1].clone();

    let new_split = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    wrap_in_split_container(
      &new_split,
      &split.clone().into(),
      &[
        window1.as_tiling_container().unwrap(),
        window2.as_tiling_container().unwrap(),
      ],
    )
    .unwrap();

    let parent_children: Vec<_> = split.children().into_iter().collect();
    assert!(!parent_children.iter().any(|c| c.id() == window1.id()));
    assert!(!parent_children.iter().any(|c| c.id() == window2.id()));

    assert_eq!(split.child_count(), initial_child_count - 1);
  }

  #[test]
  fn wrap_in_split_container_inserts_at_starting_index() {
    let state = three_children_wm_state();
    let monitor = state.monitors()[0].clone();
    let workspace = monitor.workspaces()[0].clone();

    let split = workspace
      .children()
      .into_iter()
      .next()
      .and_then(|c| c.as_split().cloned())
      .expect("Expected split");

    let children: Vec<_> = split.children().into_iter().collect();
    let window2 = children[1].clone();
    let window3 = children[2].clone();

    let new_split = SplitContainer::new(
      TilingDirection::Vertical,
      GapsConfig::default(),
    );

    wrap_in_split_container(
      &new_split,
      &split.clone().into(),
      &[
        window2.as_tiling_container().unwrap(),
        window3.as_tiling_container().unwrap(),
      ],
    )
    .unwrap();

    let parent_children: Vec<_> = split.children().into_iter().collect();
    let first_child = parent_children.into_iter().next().unwrap();
    assert_eq!(first_child.id(), children[0].id());

    let second_child = split.children().into_iter().nth(1).unwrap();
    assert_eq!(second_child.id(), new_split.id());
  }
}
