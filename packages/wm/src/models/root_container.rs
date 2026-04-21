use std::{
  cell::{Ref, RefCell, RefMut},
  collections::{HashMap, VecDeque},
  rc::Rc,
};

use anyhow::bail;
use uuid::Uuid;
use wm_common::{ContainerDto, RootContainerDto};
use wm_platform::{NativeWindow, Rect, WindowId};

use crate::{
  impl_common_getters, impl_container_debug,
  models::{
    Container, DirectionContainer, Monitor, TilingContainer,
    WindowContainer,
  },
  traits::{CommonGetters, PositionGetters},
};

/// Root node of the container tree.
#[derive(Clone)]
pub struct RootContainer(Rc<RefCell<RootContainerInner>>);

struct RootContainerInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  native_windows: HashMap<WindowId, Rc<dyn NativeWindow>>,
}

impl Default for RootContainer {
  fn default() -> Self {
    let root = RootContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      native_windows: HashMap::new(),
    };

    Self(Rc::new(RefCell::new(root)))
  }
}

impl RootContainer {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn insert_native_window(
    &self,
    window: Rc<dyn NativeWindow>,
  ) -> WindowId {
    let id = window.id();
    self.0.borrow_mut().native_windows.insert(id, window);
    id
  }

  pub fn get_native_window(
    &self,
    id: WindowId,
  ) -> Option<Rc<dyn NativeWindow>> {
    self.0.borrow().native_windows.get(&id).cloned()
  }

  pub fn remove_native_window(
    &self,
    id: WindowId,
  ) -> Option<Rc<dyn NativeWindow>> {
    self.0.borrow_mut().native_windows.remove(&id)
  }

  pub fn monitors(&self) -> Vec<Monitor> {
    self
      .children()
      .into_iter()
      .filter_map(|container| container.as_monitor().cloned())
      .collect()
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let children = self
      .children()
      .iter()
      .map(CommonGetters::to_dto)
      .try_collect()?;

    Ok(ContainerDto::Root(RootContainerDto {
      id: self.id(),
      parent_id: None,
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
    }))
  }
}

impl_container_debug!(RootContainer);
impl_common_getters!(RootContainer);

impl PositionGetters for RootContainer {
  fn to_rect(&self) -> anyhow::Result<Rect> {
    bail!("Root container does not have a position.")
  }
}
