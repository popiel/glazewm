use serde::{Deserialize, Serialize};

use super::LengthValue;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RectDelta {
  /// The delta in x-coordinates on the left of the rectangle.
  pub left: LengthValue,

  /// The delta in y-coordinates on the top of the rectangle.
  pub top: LengthValue,

  /// The delta in x-coordinates on the right of the rectangle.
  pub right: LengthValue,

  /// The delta in y-coordinates on the bottom of the rectangle.
  pub bottom: LengthValue,
}

impl RectDelta {
  #[must_use]
  pub fn new(
    left: LengthValue,
    top: LengthValue,
    right: LengthValue,
    bottom: LengthValue,
  ) -> Self {
    Self {
      left,
      top,
      right,
      bottom,
    }
  }

  /// Checks if the rectangle delta has a value greater than 1.0(px/%) for
  /// any of its sides.
  #[must_use]
  pub fn is_significant(&self) -> bool {
    self.bottom.amount > 1.0
      || self.top.amount > 1.0
      || self.left.amount > 1.0
      || self.right.amount > 1.0
  }

  /// Creates a new `RectDelta` with all sides set to 0px.
  #[must_use]
  pub fn zero() -> Self {
    Self::new(
      LengthValue::from_px(0),
      LengthValue::from_px(0),
      LengthValue::from_px(0),
      LengthValue::from_px(0),
    )
  }

  /// Gets the inverse of this delta by negating all values.
  ///
  /// Returns a new `RectDelta` instance.
  #[must_use]
  pub fn inverse(&self) -> Self {
    RectDelta::new(
      LengthValue {
        amount: -self.left.amount,
        unit: self.left.unit.clone(),
      },
      LengthValue {
        amount: -self.top.amount,
        unit: self.top.unit.clone(),
      },
      LengthValue {
        amount: -self.right.amount,
        unit: self.right.unit.clone(),
      },
      LengthValue {
        amount: -self.bottom.amount,
        unit: self.bottom.unit.clone(),
      },
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::LengthUnit;

  #[test]
  fn is_significant_returns_true_when_any_side_greater_than_one() {
    let delta = RectDelta::new(
      LengthValue::from_px(2),
      LengthValue::from_px(0),
      LengthValue::from_px(0),
      LengthValue::from_px(0),
    );
    assert!(delta.is_significant());
  }

  #[test]
  fn is_significant_returns_false_when_all_sides_one_or_less() {
    let delta = RectDelta::new(
      LengthValue::from_px(1),
      LengthValue::from_px(1),
      LengthValue::from_px(1),
      LengthValue::from_px(1),
    );
    assert!(!delta.is_significant());
  }

  #[test]
  fn inverse_negates_all_values() {
    let delta = RectDelta::new(
      LengthValue::from_px(10),
      LengthValue::from_px(20),
      LengthValue::from_px(30),
      LengthValue::from_px(40),
    );
    let inverted = delta.inverse();
    assert_eq!(inverted.left.amount, -10.0);
    assert_eq!(inverted.top.amount, -20.0);
    assert_eq!(inverted.right.amount, -30.0);
    assert_eq!(inverted.bottom.amount, -40.0);
  }
}
