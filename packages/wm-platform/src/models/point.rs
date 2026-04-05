/// Represents an x-y coordinate.
#[derive(Debug, Clone)]
pub struct Point {
  pub x: i32,
  pub y: i32,
}

impl Point {
  /// Calculates the Euclidean distance between this point and another
  /// point.
  #[must_use]
  pub fn distance_between(&self, other: &Point) -> f32 {
    let dx = self.x - other.x;
    let dy = self.y - other.y;

    #[allow(clippy::cast_precision_loss)]
    ((dx * dx + dy * dy) as f32).sqrt()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn distance_between_same_point_is_zero() {
    let p1 = Point { x: 10, y: 20 };
    assert_eq!(p1.distance_between(&p1), 0.0);
  }

  #[test]
  fn distance_between_horizontal() {
    let p1 = Point { x: 0, y: 0 };
    let p2 = Point { x: 3, y: 0 };
    assert_eq!(p1.distance_between(&p2), 3.0);
  }

  #[test]
  fn distance_between_vertical() {
    let p1 = Point { x: 0, y: 0 };
    let p2 = Point { x: 0, y: 4 };
    assert_eq!(p1.distance_between(&p2), 4.0);
  }

  #[test]
  fn distance_between_diagonal() {
    let p1 = Point { x: 0, y: 0 };
    let p2 = Point { x: 3, y: 4 };
    assert_eq!(p1.distance_between(&p2), 5.0);
  }

  #[test]
  fn distance_between_negative_coordinates() {
    let p1 = Point { x: -5, y: -5 };
    let p2 = Point { x: 5, y: 5 };
    assert!((p1.distance_between(&p2) - 14.142).abs() < 0.01);
  }
}
