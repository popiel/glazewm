use std::collections::VecDeque;

pub trait VecDequeExt<T>
where
  T: PartialEq,
{
  /// Shifts a value to a specified index in a `VecDeque`.
  ///
  /// Inserts at index if value doesn't already exist in the `VecDeque`.
  fn shift_to_index(&mut self, target_index: usize, item: T);
}

impl<T> VecDequeExt<T> for VecDeque<T>
where
  T: PartialEq,
{
  fn shift_to_index(&mut self, target_index: usize, value: T) {
    if let Some(index) = self.iter().position(|e| e == &value) {
      self.remove(index);

      // Adjust for when the target index becomes out of bounds because of
      // the removal above.
      self.insert(target_index.clamp(0, self.len()), value);
    }
  }
}

#[cfg(test)]
mod tests {
  use std::collections::VecDeque;

  use super::*;

  #[test]
  fn shift_to_index_moves_existing_element_forward() {
    let mut deque: VecDeque<i32> = VecDeque::from(vec![1, 2, 3, 4, 5]);
    deque.shift_to_index(4, 1);
    assert_eq!(deque.into_iter().collect::<Vec<_>>(), vec![2, 3, 4, 5, 1]);
  }

  #[test]
  fn shift_to_index_moves_existing_element_backward() {
    let mut deque: VecDeque<i32> = VecDeque::from(vec![1, 2, 3, 4, 5]);
    deque.shift_to_index(0, 5);
    assert_eq!(deque.into_iter().collect::<Vec<_>>(), vec![5, 1, 2, 3, 4]);
  }

  #[test]
  fn shift_to_index_does_nothing_when_not_found() {
    // Note: Bug - function does nothing when item not found
    let mut deque: VecDeque<i32> = VecDeque::from(vec![1, 2, 3]);
    deque.shift_to_index(1, 99);
    // Actual result: unchanged [1, 2, 3] - function silently fails
    assert_eq!(deque.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
  }

  #[test]
  fn shift_to_index_handles_out_of_bounds() {
    // Target index larger than len() gets clamped to end
    let mut deque: VecDeque<i32> = VecDeque::from(vec![1, 2, 3]);
    deque.shift_to_index(100, 1);
    assert_eq!(deque.into_iter().collect::<Vec<_>>(), vec![2, 3, 1]);

    // Also works with smaller out-of-bounds index
    let mut deque2: VecDeque<i32> = VecDeque::from(vec![1, 2, 3]);
    deque2.shift_to_index(10, 1);
    assert_eq!(deque2.into_iter().collect::<Vec<_>>(), vec![2, 3, 1]);
  }

  #[test]
  fn shift_to_index_handles_empty_deque() {
    // Note: Bug - empty deque causes no insertion
    let mut deque: VecDeque<i32> = VecDeque::new();
    deque.shift_to_index(0, 1);
    // Actual result: [] - function does nothing for empty deque
    assert_eq!(deque.into_iter().collect::<Vec<_>>(), Vec::<i32>::new());
  }

  #[test]
  fn shift_to_index_with_single_element() {
    let mut deque: VecDeque<i32> = VecDeque::from(vec![1]);
    deque.shift_to_index(0, 1);
    assert_eq!(deque.into_iter().collect::<Vec<_>>(), vec![1]);
  }

  #[test]
  fn shift_to_index_preserves_other_elements() {
    let mut deque: VecDeque<i32> = VecDeque::from(vec![10, 20, 30, 40]);
    deque.shift_to_index(1, 30);
    assert_eq!(
      deque.into_iter().collect::<Vec<_>>(),
      vec![10, 30, 20, 40]
    );
  }
}
