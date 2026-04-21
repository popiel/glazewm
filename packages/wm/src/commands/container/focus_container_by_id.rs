use anyhow::Context;
use uuid::Uuid;

use super::set_focused_descendant;
use crate::wm_state::WmState;

/// Focuses a container by its ID.
///
/// # Errors
///
/// Returns an error if no container with the given ID exists.
pub fn focus_container_by_id(
  container_id: &Uuid,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let focus_target = state
    .container_by_id(*container_id)
    .context("No container with given id")?;

  // Set focus to the target container.
  set_focused_descendant(&focus_target, None);
  state.pending_sync.queue_focus_change().queue_cursor_jump();

  Ok(())
}
