use serde::{Deserialize, Serialize};

use crate::{Direction, LengthValue, Point, RectDelta};

#[derive(Debug, Deserialize, Clone, Serialize, Eq, PartialEq)]
pub enum Corner {
  TopLeft,
  TopRight,
  BottomLeft,
  BottomRight,
}

#[derive(Debug, Deserialize, Clone, Serialize, Eq, PartialEq)]
pub struct Rect {
  /// X-coordinate of the left edge of the rectangle.
  pub left: i32,

  /// Y-coordinate of the top edge of the rectangle.
  pub top: i32,

  /// X-coordinate of the right edge of the rectangle.
  pub right: i32,

  /// Y-coordinate of the bottom edge of the rectangle.
  pub bottom: i32,
}

impl Rect {
  /// Creates a new `Rect` instance from the coordinates of its left, top,
  /// right, and bottom edges.
  #[must_use]
  pub fn from_ltrb(left: i32, top: i32, right: i32, bottom: i32) -> Self {
    Self {
      left,
      top,
      right,
      bottom,
    }
  }

  /// Creates a new `Rect` instance from its X/Y coordinates and size.
  #[must_use]
  pub fn from_xy(x: i32, y: i32, width: i32, height: i32) -> Self {
    Self {
      left: x,
      top: y,
      right: x + width,
      bottom: y + height,
    }
  }

  #[must_use]
  pub fn x(&self) -> i32 {
    self.left
  }

  #[must_use]
  pub fn y(&self) -> i32 {
    self.top
  }

  #[must_use]
  pub fn width(&self) -> i32 {
    self.right - self.left
  }

  #[must_use]
  pub fn height(&self) -> i32 {
    self.bottom - self.top
  }

  #[must_use]
  pub fn translate_to_coordinates(&self, x: i32, y: i32) -> Self {
    Self::from_xy(x, y, self.width(), self.height())
  }

  #[must_use]
  pub fn translate_to_center(&self, outer_rect: &Rect) -> Self {
    Self::translate_to_coordinates(
      self,
      outer_rect.left + (outer_rect.width() / 2) - (self.width() / 2),
      outer_rect.top + (outer_rect.height() / 2) - (self.height() / 2),
    )
  }

  #[must_use]
  pub fn translate_in_direction(
    &self,
    direction: &Direction,
    distance: i32,
  ) -> Rect {
    let (delta_x, delta_y) = match direction {
      Direction::Up => (0, -distance),
      Direction::Down => (0, distance),
      Direction::Left => (-distance, 0),
      Direction::Right => (distance, 0),
    };

    Self::from_xy(
      self.x() + delta_x,
      self.y() + delta_y,
      self.width(),
      self.height(),
    )
  }

  /// Returns a new `Rect` that is clamped within the bounds of the given
  /// outer rectangle. Attempts to preserve the width and height of the
  /// original rectangle.
  #[must_use]
  pub fn clamp(&self, outer_rect: &Rect) -> Self {
    Self::from_xy(
      self.left.max(outer_rect.left),
      self.top.max(outer_rect.top),
      self.width().min(outer_rect.width()),
      self.height().min(outer_rect.height()),
    )
  }

  #[must_use]
  pub fn clamp_size(&self, width: i32, height: i32) -> Self {
    Self::from_xy(
      self.x(),
      self.y(),
      self.width().min(width),
      self.height().min(height),
    )
  }

  #[must_use]
  pub fn center_point(&self) -> Point {
    Point {
      x: self.left + (self.width() / 2),
      y: self.top + (self.height() / 2),
    }
  }

  #[must_use]
  pub fn corner(&self, corner: &Corner) -> Point {
    match corner {
      Corner::TopLeft => Point {
        x: self.left,
        y: self.top,
      },
      Corner::TopRight => Point {
        x: self.right,
        y: self.top,
      },
      Corner::BottomLeft => Point {
        x: self.left,
        y: self.bottom,
      },
      Corner::BottomRight => Point {
        x: self.right,
        y: self.bottom,
      },
    }
  }

  /// Gets the delta between this rect and another rect.
  #[must_use]
  pub fn delta(&self, other: &Rect) -> RectDelta {
    RectDelta {
      left: LengthValue::from_px(other.left - self.left),
      top: LengthValue::from_px(other.top - self.top),
      right: LengthValue::from_px(self.right - other.right),
      bottom: LengthValue::from_px(self.bottom - other.bottom),
    }
  }

  #[must_use]
  pub fn apply_delta(
    &self,
    delta: &RectDelta,
    scale_factor: Option<f32>,
  ) -> Self {
    Self::from_ltrb(
      self.left - delta.left.to_px(self.width(), scale_factor),
      self.top - delta.top.to_px(self.height(), scale_factor),
      self.right + delta.right.to_px(self.width(), scale_factor),
      self.bottom + delta.bottom.to_px(self.height(), scale_factor),
    )
  }

  // Gets the amount of overlap between the x-coordinates of the two rects.
  #[must_use]
  pub fn x_overlap(&self, other: &Rect) -> i32 {
    self.right.min(other.right) - self.x().max(other.x())
  }

  // Gets the amount of overlap between the y-coordinates of the two rects.
  #[must_use]
  pub fn y_overlap(&self, other: &Rect) -> i32 {
    self.bottom.min(other.bottom) - self.y().max(other.y())
  }

  /// Gets the intersection area of this rect and another rect.
  #[must_use]
  pub fn intersection_area(&self, other: &Rect) -> i32 {
    let x_overlap = self.x_overlap(other);
    let y_overlap = self.y_overlap(other);

    if x_overlap > 0 && y_overlap > 0 {
      x_overlap * y_overlap
    } else {
      0
    }
  }

  #[must_use]
  pub fn contains_point(&self, point: &Point) -> bool {
    let is_in_x = point.x >= self.left && point.x <= self.right;
    let is_in_y = point.y >= self.top && point.y <= self.bottom;
    is_in_x && is_in_y
  }

  /// Gets whether this rect fully encloses another rect.
  #[must_use]
  pub fn contains_rect(&self, other: &Rect) -> bool {
    self.left <= other.left
      && self.top <= other.top
      && self.right >= other.right
      && self.bottom >= other.bottom
  }

  /// Creates a new rect that is inset by the given amount of pixels on all
  /// sides.
  ///
  /// The `inset_px` can be a positive number to create a smaller rect
  /// (inset), or a negative number to create a larger rect (outset).
  #[must_use]
  pub fn inset(&self, inset_px: i32) -> Self {
    Self::from_ltrb(
      self.left + inset_px,
      self.top + inset_px,
      self.right - inset_px,
      self.bottom - inset_px,
    )
  }

  #[must_use]
  pub fn distance_to_point(&self, point: &Point) -> f32 {
    let dx = (self.x() - point.x)
      .abs()
      .max((self.x() + self.width() - point.x).abs());

    let dy = (self.y() - point.y)
      .abs()
      .max((self.y() + self.height() - point.y).abs());

    #[allow(clippy::cast_precision_loss)]
    ((dx * dx + dy * dy) as f32).sqrt()
  }

  /// Returns the union of this rect and another rect.
  ///
  /// The union is the smallest rect that contains both rects, taking the
  /// minimum left/top and maximum right/bottom coordinates.
  #[must_use]
  pub fn union(&self, other: &Rect) -> Self {
    Self::from_ltrb(
      self.left.min(other.left),
      self.top.min(other.top),
      self.right.max(other.right),
      self.bottom.max(other.bottom),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Direction;

  #[test]
  fn intersection_area_full_overlap() {
    let r1 = Rect::from_xy(0, 0, 100, 100);
    let r2 = Rect::from_xy(0, 0, 100, 100);
    assert_eq!(r1.intersection_area(&r2), 10000);
  }

  #[test]
  fn intersection_area_partial_overlap() {
    let r1 = Rect::from_xy(0, 0, 100, 100);
    let r2 = Rect::from_xy(50, 50, 100, 100);
    assert_eq!(r1.intersection_area(&r2), 2500);
  }

  #[test]
  fn intersection_area_no_overlap() {
    let r1 = Rect::from_xy(0, 0, 100, 100);
    let r2 = Rect::from_xy(200, 200, 100, 100);
    assert_eq!(r1.intersection_area(&r2), 0);
  }

  #[test]
  fn intersection_area_edges_touching() {
    let r1 = Rect::from_xy(0, 0, 100, 100);
    let r2 = Rect::from_xy(100, 0, 100, 100);
    assert_eq!(r1.intersection_area(&r2), 0);
  }

  #[test]
  fn contains_point_inside_rect() {
    let rect = Rect::from_xy(0, 0, 100, 100);
    let point = Point { x: 50, y: 50 };
    assert!(rect.contains_point(&point));
  }

  #[test]
  fn contains_point_on_edge() {
    let rect = Rect::from_xy(0, 0, 100, 100);
    let point = Point { x: 0, y: 0 };
    assert!(rect.contains_point(&point));
  }

  #[test]
  fn contains_point_outside_rect() {
    let rect = Rect::from_xy(0, 0, 100, 100);
    let point = Point { x: 150, y: 50 };
    assert!(!rect.contains_point(&point));
  }

  #[test]
  fn contains_rect_full_containment() {
    let outer = Rect::from_xy(0, 0, 100, 100);
    let inner = Rect::from_xy(10, 10, 50, 50);
    assert!(outer.contains_rect(&inner));
  }

  #[test]
  fn contains_rect_partial_overlap() {
    let outer = Rect::from_xy(0, 0, 100, 100);
    let other = Rect::from_xy(50, 50, 100, 100);
    assert!(!outer.contains_rect(&other));
  }

  #[test]
  fn contains_rect_same_rect() {
    let rect = Rect::from_xy(0, 0, 100, 100);
    assert!(rect.contains_rect(&rect));
  }

  #[test]
  fn union_two_disjoint_rects() {
    let r1 = Rect::from_xy(0, 0, 50, 50);
    let r2 = Rect::from_xy(100, 100, 50, 50);
    let result = r1.union(&r2);
    assert_eq!(result.left, 0);
    assert_eq!(result.top, 0);
    assert_eq!(result.right, 150);
    assert_eq!(result.bottom, 150);
  }

  #[test]
  fn union_two_overlapping_rects() {
    let r1 = Rect::from_xy(0, 0, 100, 100);
    let r2 = Rect::from_xy(50, 50, 100, 100);
    let result = r1.union(&r2);
    assert_eq!(result.left, 0);
    assert_eq!(result.top, 0);
    assert_eq!(result.right, 150);
    assert_eq!(result.bottom, 150);
  }

  #[test]
  fn clamp_rect_within_larger_rect() {
    let inner = Rect::from_xy(50, 50, 100, 100);
    let outer = Rect::from_xy(0, 0, 200, 200);
    let result = inner.clamp(&outer);
    assert_eq!(result.left, 50);
    assert_eq!(result.top, 50);
    assert_eq!(result.width(), 100);
    assert_eq!(result.height(), 100);
  }

  #[test]
  fn clamp_rect_extends_beyond_bounds() {
    // When inner rect extends beyond outer, clamp repositions it to outer
    // bounds but preserves the inner rect's dimensions (not shrunk to
    // fit)
    let inner = Rect::from_xy(-50, -50, 100, 100);
    let outer = Rect::from_xy(0, 0, 200, 200);
    let result = inner.clamp(&outer);
    assert_eq!(result.left, 0);
    assert_eq!(result.top, 0);
    // The clamp function preserves width/height of inner rect
    assert_eq!(result.width(), 100);
    assert_eq!(result.height(), 100);
  }

  #[test]
  fn inset_positive_reduces_size() {
    let rect = Rect::from_xy(10, 10, 100, 100); // left=10, top=10, right=110, bottom=110
    let result = rect.inset(10);
    assert_eq!(result.left, 20);
    assert_eq!(result.top, 20);
    assert_eq!(result.right, 100);
    assert_eq!(result.bottom, 100);
  }

  #[test]
  fn inset_negative_increases_size() {
    let rect = Rect::from_xy(10, 10, 100, 100);
    let result = rect.inset(-10);
    assert_eq!(result.left, 0);
    assert_eq!(result.top, 0);
    assert_eq!(result.right, 120);
    assert_eq!(result.bottom, 120);
  }

  #[test]
  fn translate_in_direction_up() {
    let rect = Rect::from_xy(50, 50, 100, 100);
    let result = rect.translate_in_direction(&Direction::Up, 10);
    assert_eq!(result.x(), 50);
    assert_eq!(result.y(), 40);
  }

  #[test]
  fn translate_in_direction_down() {
    let rect = Rect::from_xy(50, 50, 100, 100);
    let result = rect.translate_in_direction(&Direction::Down, 10);
    assert_eq!(result.x(), 50);
    assert_eq!(result.y(), 60);
  }

  #[test]
  fn translate_in_direction_left() {
    let rect = Rect::from_xy(50, 50, 100, 100);
    let result = rect.translate_in_direction(&Direction::Left, 10);
    assert_eq!(result.x(), 40);
    assert_eq!(result.y(), 50);
  }

  #[test]
  fn translate_in_direction_right() {
    let rect = Rect::from_xy(50, 50, 100, 100);
    let result = rect.translate_in_direction(&Direction::Right, 10);
    assert_eq!(result.x(), 60);
    assert_eq!(result.y(), 50);
  }

  #[test]
  fn translate_to_center_centers_rect() {
    let inner = Rect::from_xy(0, 0, 50, 50);
    let outer = Rect::from_xy(0, 0, 200, 200);
    let result = inner.translate_to_center(&outer);
    assert_eq!(result.x(), 75);
    assert_eq!(result.y(), 75);
  }

  #[test]
  fn width_and_height_calculation() {
    let rect = Rect::from_xy(10, 20, 100, 50);
    assert_eq!(rect.width(), 100);
    assert_eq!(rect.height(), 50);
  }

  #[test]
  fn x_and_y_return_left_top() {
    let rect = Rect::from_xy(10, 20, 100, 50);
    assert_eq!(rect.x(), 10);
    assert_eq!(rect.y(), 20);
  }

  #[test]
  fn from_ltrb_creates_correct_rect() {
    let rect = Rect::from_ltrb(10, 20, 110, 70);
    assert_eq!(rect.left, 10);
    assert_eq!(rect.top, 20);
    assert_eq!(rect.right, 110);
    assert_eq!(rect.bottom, 70);
    assert_eq!(rect.width(), 100);
    assert_eq!(rect.height(), 50);
  }

  #[test]
  fn corner_top_left() {
    let rect = Rect::from_xy(10, 20, 100, 50);
    let corner = rect.corner(&Corner::TopLeft);
    assert_eq!(corner.x, 10);
    assert_eq!(corner.y, 20);
  }

  #[test]
  fn corner_top_right() {
    let rect = Rect::from_xy(10, 20, 100, 50);
    let corner = rect.corner(&Corner::TopRight);
    assert_eq!(corner.x, 110);
    assert_eq!(corner.y, 20);
  }

  #[test]
  fn corner_bottom_left() {
    let rect = Rect::from_xy(10, 20, 100, 50);
    let corner = rect.corner(&Corner::BottomLeft);
    assert_eq!(corner.x, 10);
    assert_eq!(corner.y, 70);
  }

  #[test]
  fn corner_bottom_right() {
    let rect = Rect::from_xy(10, 20, 100, 50);
    let corner = rect.corner(&Corner::BottomRight);
    assert_eq!(corner.x, 110);
    assert_eq!(corner.y, 70);
  }

  #[test]
  fn center_point_calculation() {
    let rect = Rect::from_xy(0, 0, 100, 100);
    let center = rect.center_point();
    assert_eq!(center.x, 50);
    assert_eq!(center.y, 50);
  }
}
