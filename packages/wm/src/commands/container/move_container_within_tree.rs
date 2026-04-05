use anyhow::Context;
use wm_common::{VecDequeExt, WmEvent};

use super::{
  attach_container, detach_container, flatten_child_split_containers,
  set_focused_descendant,
};
use crate::{models::Container, traits::CommonGetters, wm_state::WmState};

/// Move a container to a new location in the tree. This detaches the
/// container from its current parent and attaches it to the new parent at
/// the specified index.
///
/// If this container is a tiling container, its siblings are resized on
/// detach, and the container is sized to the default tiling size with its
/// new siblings. No changes to the container's tiling size are made if
/// its parent stays the same.
///
/// This will flatten any redundant split containers after moving the
/// container, which can cause the target parent to become detached. For
/// example, in the layout V[1 H[2]] where container 1 is moved down, the
/// parent gets removed resulting in V[1 2].
pub fn move_container_within_tree(
  container_to_move: &Container,
  target_parent: &Container,
  target_index: usize,
  state: &WmState,
) -> anyhow::Result<()> {
  // Create iterator of parent, grandparent, and great-grandparent.
  let ancestors =
    container_to_move.ancestors().take(3).collect::<Vec<_>>();

  // Get lowest common ancestor (LCA) between `container_to_move` and
  // `target_parent`. This could be the `target_parent` itself.
  let lowest_common_ancestor =
    lowest_common_ancestor(container_to_move, target_parent)
      .context("No common ancestor between containers.")?;

  // If the container is already a child of the target parent, then shift
  // it to the target index.
  if container_to_move.parent().context("No parent.")? == *target_parent {
    target_parent
      .borrow_children_mut()
      .shift_to_index(target_index, container_to_move.clone());

    if container_to_move.has_focus(None) {
      state.emit_event(WmEvent::FocusedContainerMoved {
        focused_container: container_to_move.to_dto()?,
      });
    }

    return Ok(());
  }

  // Handle case where target parent is the LCA. For example, when swapping
  // sibling containers or moving a container to a direct ancestor.
  if *target_parent == lowest_common_ancestor {
    return move_to_lowest_common_ancestor(
      container_to_move,
      &lowest_common_ancestor,
      target_index,
      state,
    );
  }

  // Get ancestor of `container_to_move` that is a direct child of the LCA.
  // This could be the `container_to_move` itself.
  let container_to_move_ancestor = container_to_move
    .self_and_ancestors()
    .find(|ancestor| {
      ancestor.parent() == Some(lowest_common_ancestor.clone())
    })
    .context("Failed to get ancestor of container to move.")?;

  // Likewise, get ancestor of `target_parent` that is a direct child of
  // the LCA.
  let target_parent_ancestor = target_parent
    .self_and_ancestors()
    .find(|ancestor| {
      ancestor.parent() == Some(lowest_common_ancestor.clone())
    })
    .context("Failed to get ancestor of target parent.")?;

  // Get whether the container is the focused descendant in its original
  // subtree from the LCA.
  let is_focused_descendant = *container_to_move
    == container_to_move_ancestor
    || container_to_move
      .has_focus(Some(container_to_move_ancestor.clone()));

  // Get whether the ancestor of `container_to_move` appears before
  // `target_parent`'s ancestor in the child focus order of the LCA.
  let original_focus_index = container_to_move_ancestor.focus_index();
  let is_subtree_focused =
    original_focus_index < target_parent_ancestor.focus_index();

  detach_container(container_to_move.clone())?;
  attach_container(
    &container_to_move.clone(),
    &target_parent.clone(),
    Some(target_index),
  )?;

  // Set `container_to_move` as focused descendant within target subtree if
  // its original subtree had focus more recently (even if the container is
  // not the last focused within that subtree).
  if is_subtree_focused {
    set_focused_descendant(
      container_to_move,
      Some(&target_parent_ancestor),
    );
  }

  // If the focused descendant is moved to the targets subtree, then the
  // target's ancestor should be placed before the original ancestor in
  // LCA's child focus order.
  if is_focused_descendant && is_subtree_focused {
    lowest_common_ancestor
      .borrow_child_focus_order_mut()
      .shift_to_index(original_focus_index, target_parent_ancestor.id());
  }

  // After moving the container, flatten any redundant split containers.
  // For example, in the layout V[1 H[2]] where container 1 is moved down
  // to become V[H[1 2]], this will then need to be flattened to V[1 2].
  for ancestor in ancestors.iter().rev() {
    flatten_child_split_containers(ancestor)?;
  }

  if container_to_move.has_focus(None) {
    state.emit_event(WmEvent::FocusedContainerMoved {
      focused_container: container_to_move.to_dto()?,
    });
  }

  Ok(())
}

fn move_to_lowest_common_ancestor(
  container_to_move: &Container,
  lowest_common_ancestor: &Container,
  target_index: usize,
  state: &WmState,
) -> anyhow::Result<()> {
  // Keep reference to focus index of container's ancestor in LCA's child
  // focus order.
  let original_focus_index = container_to_move
    .self_and_ancestors()
    .find(|ancestor| {
      ancestor.parent() == Some(lowest_common_ancestor.clone())
    })
    .map(|ancestor| ancestor.focus_index())
    .context("Failed to get focus index of container's ancestor.")?;

  detach_container(container_to_move.clone())?;

  attach_container(
    &container_to_move.clone(),
    &lowest_common_ancestor.clone(),
    Some(target_index),
  )?;

  lowest_common_ancestor
    .borrow_child_focus_order_mut()
    .shift_to_index(original_focus_index, container_to_move.id());

  if container_to_move.has_focus(None) {
    state.emit_event(WmEvent::FocusedContainerMoved {
      focused_container: container_to_move.to_dto()?,
    });
  }

  Ok(())
}

/// Gets the lowest container in the tree that has both `container_a` and
/// `container_b` as descendants.
pub fn lowest_common_ancestor(
  container_a: &Container,
  container_b: &Container,
) -> Option<Container> {
  let mut ancestor_a = Some(container_a.clone());

  // Traverse upwards from container A.
  while let Some(current_ancestor_a) = ancestor_a {
    let mut ancestor_b = Some(container_b.clone());

    // Traverse upwards from container B.
    while let Some(current_ancestor_b) = ancestor_b {
      if current_ancestor_a == current_ancestor_b {
        return Some(current_ancestor_a);
      }

      ancestor_b = current_ancestor_b.parent();
    }

    ancestor_a = current_ancestor_a.parent();
  }

  None
}

#[cfg(test)]
mod tests {
  use wm_common::{GapsConfig, TilingDirection};

  use super::*;
  use crate::{
    commands::container::attach_container,
    models::{SplitContainer, TilingWindow},
    tests::{three_children_wm_state, TestWmStateBuilder},
    traits::{CommonGetters, TilingSizeGetters},
  };

  #[test]
  fn lowest_common_ancestor_same_container() {
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
    let window = children[0].clone();

    let lca = lowest_common_ancestor(&window, &window);

    assert_eq!(lca.map(|c| c.id()), Some(window.id()));
  }

  #[test]
  fn lowest_common_ancestor_siblings() {
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

    let lca = lowest_common_ancestor(&window1, &window2);

    assert_eq!(lca.map(|c| c.id()), Some(split.id()));
  }

  #[test]
  fn lowest_common_ancestor_parent_child() {
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
    let window = children[0].clone();

    let lca = lowest_common_ancestor(&split.clone().into(), &window);

    assert_eq!(lca.map(|c| c.id()), Some(split.id()));
  }

  #[test]
  fn lowest_common_ancestor_grandparent() {
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
    let window = children[0].clone();

    let lca = lowest_common_ancestor(&workspace.clone().into(), &window);

    assert_eq!(lca.map(|c| c.id()), Some(workspace.id()));
  }

  #[test]
  fn lowest_common_ancestor_nested_splits() {
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
    let window3 = TilingWindow::new_test("W3");

    outer.set_tiling_size(1.0);
    inner.set_tiling_size(0.5);
    window1.set_tiling_size(0.5);
    window2.set_tiling_size(0.5);
    window3.set_tiling_size(0.5);

    attach_container(&inner.clone().into(), &outer.clone().into(), None)
      .unwrap();
    attach_container(&window3.clone().into(), &outer.clone().into(), None)
      .unwrap();
    attach_container(&window1.clone().into(), &inner.clone().into(), None)
      .unwrap();
    attach_container(&window2.clone().into(), &inner.clone().into(), None)
      .unwrap();

    let lca = lowest_common_ancestor(
      &window1.clone().into(),
      &window3.clone().into(),
    );
    assert_eq!(lca.map(|c| c.id()), Some(outer.id()));

    let lca_nested = lowest_common_ancestor(
      &window1.clone().into(),
      &window2.clone().into(),
    );
    assert_eq!(lca_nested.map(|c| c.id()), Some(inner.id()));
  }

  #[test]
  fn lowest_common_ancestor_different_branches() {
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
    let window3 = children[2].clone();

    let lca = lowest_common_ancestor(&window1, &window3);

    assert_eq!(lca.map(|c| c.id()), Some(split.id()));
  }

  #[test]
  fn lowest_common_ancestor_no_common_ancestor() {
    let window1 = TilingWindow::new_test("W1");
    let window2 = TilingWindow::new_test("W2");

    let lca = lowest_common_ancestor(
      &window1.clone().into(),
      &window2.clone().into(),
    );

    assert!(lca.is_none());
  }

  #[test]
  fn move_container_within_tree_reorders_same_parent() {
    let (state, _config) = TestWmStateBuilder::new()
      .with_monitor("DP-1")
      .with_workspace("1")
      .with_tiling_window("W1")
      .with_tiling_window("W2")
      .with_tiling_window("W3")
      .with_focused(0, 0)
      .build();

    let monitor = state.monitors().into_iter().next().unwrap();
    let workspace = monitor.displayed_workspace().unwrap();
    let split = workspace.children().into_iter().next().unwrap();

    let children: Vec<_> = split.children().into_iter().collect();
    let window1 = children[0].clone();

    move_container_within_tree(&window1, &split, 2, &state).unwrap();

    let children_after: Vec<_> = split.children().into_iter().collect();
    assert_eq!(
      children_after[0].id(),
      children[1].id(),
      "W2 should now be first"
    );
    assert_eq!(
      children_after[1].id(),
      children[2].id(),
      "W3 should be second"
    );
    assert_eq!(
      children_after[2].id(),
      window1.id(),
      "W1 should now be last"
    );
  }

  #[test]
  fn move_container_within_tree_moves_to_different_parent() {
    use wm_common::WorkspaceConfig;

    use crate::models::Workspace;

    let (state, _config) = TestWmStateBuilder::new()
      .with_monitor("DP-1")
      .with_workspace("1")
      .with_tiling_window("W1")
      .with_tiling_window("W2")
      .with_tiling_window("W3")
      .build();

    let monitor = state.monitors().into_iter().next().unwrap();
    let ws2_config = WorkspaceConfig {
      name: "2".to_string(),
      display_name: None,
      bind_to_monitor: None,
      keep_alive: false,
    };
    let ws2 = Workspace::new(
      ws2_config,
      wm_common::GapsConfig::default(),
      wm_common::TilingDirection::Horizontal,
    );

    attach_container(&ws2.clone().into(), &monitor.clone().into(), None)
      .unwrap();

    let workspace1 = monitor.displayed_workspace().unwrap();
    let split1 = workspace1.children().into_iter().next().unwrap();

    let children1: Vec<_> = split1.children().into_iter().collect();
    let window1 = children1[0].clone();

    move_container_within_tree(&window1, &ws2.clone().into(), 0, &state)
      .unwrap();

    let ws1_window_count = workspace1
      .descendants()
      .filter(|c| matches!(c, crate::models::Container::TilingWindow(_)))
      .count();
    assert_eq!(
      ws1_window_count, 2,
      "Workspace 1 should have 2 windows left"
    );

    let ws2_children: Vec<_> = ws2.children().into_iter().collect();

    assert_eq!(ws2_children.len(), 1, "Workspace 2 should have 1 window");
    assert_eq!(
      ws2_children[0].id(),
      window1.id(),
      "W1 should be in workspace 2"
    );
  }
}
