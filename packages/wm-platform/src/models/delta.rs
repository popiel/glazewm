use std::str::FromStr;

use serde::Serialize;

/// A wrapper that indicates a value should be interpreted as a delta
/// (relative change).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Delta<T> {
  pub inner: T,
  pub is_negative: bool,
}

impl<T: FromStr<Err = crate::ParseError>> FromStr for Delta<T> {
  type Err = crate::ParseError;

  fn from_str(unparsed: &str) -> Result<Self, crate::ParseError> {
    let unparsed = unparsed.trim();

    let (raw, is_negative) = match unparsed.chars().next() {
      Some('+') => (&unparsed[1..], false),
      Some('-') => (&unparsed[1..], true),
      // No sign is interpreted as positive.
      _ => (unparsed, false),
    };

    if raw.is_empty() {
      return Err(crate::ParseError::Delta(unparsed.to_string()));
    }

    let inner = T::from_str(raw)?;

    Ok(Self { inner, is_negative })
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::*;
  use crate::OpacityValue;

  #[test]
  fn delta_parses_positive_number() {
    let result = Delta::<OpacityValue>::from_str("+0.5").unwrap();
    assert!((result.inner.0 - 0.5).abs() < 0.001);
    assert!(!result.is_negative);
  }

  #[test]
  fn delta_parses_negative_number() {
    let result = Delta::<OpacityValue>::from_str("-0.25").unwrap();
    assert!((result.inner.0 - 0.25).abs() < 0.001);
    assert!(result.is_negative);
  }

  #[test]
  fn delta_parses_unsigned_number() {
    let result = Delta::<OpacityValue>::from_str("0.75").unwrap();
    assert!((result.inner.0 - 0.75).abs() < 0.001);
    assert!(!result.is_negative);
  }

  #[test]
  fn delta_parses_with_whitespace() {
    let result = Delta::<OpacityValue>::from_str("  +0.25  ").unwrap();
    assert!((result.inner.0 - 0.25).abs() < 0.001);
    assert!(!result.is_negative);
  }

  #[test]
  fn delta_fails_on_empty_string() {
    let result = Delta::<OpacityValue>::from_str("");
    assert!(result.is_err());
  }

  #[test]
  fn delta_fails_on_sign_only() {
    let result = Delta::<OpacityValue>::from_str("-");
    assert!(result.is_err());
  }

  #[test]
  fn delta_parses_percentage_value() {
    let result = Delta::<OpacityValue>::from_str("+50%").unwrap();
    assert!((result.inner.0 - 0.5).abs() < 0.001);
    assert!(!result.is_negative);
  }

  #[test]
  fn delta_parses_negative_percentage() {
    let result = Delta::<OpacityValue>::from_str("-25%").unwrap();
    assert!((result.inner.0 - 0.25).abs() < 0.001);
    assert!(result.is_negative);
  }
}
