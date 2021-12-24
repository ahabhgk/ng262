use std::ops::Deref;

use super::string::JsString;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct JsNumber(f64);

impl Deref for JsNumber {
  type Target = f64;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl JsNumber {
  /// https://tc39.es/ecma262/#sec-numeric-types-number-equal
  pub fn equal(x: &f64, y: &f64) -> bool {
    // 1. If x is NaN, return false.
    // 2. If y is NaN, return false.
    // 3. If x is the same Number value as y, return true.
    // 4. If x is +0𝔽 and y is -0𝔽, return true.
    // 5. If x is -0𝔽 and y is +0𝔽, return true.
    // 6. Return false.
    x == y
  }

  /// https://tc39.es/ecma262/#sec-numeric-types-number-sameValue
  pub fn same_value(x: &f64, y: &f64) -> bool {
    // 1. If x is NaN and y is NaN, return true.
    if x.is_nan() && y.is_nan() {
      return true;
    }
    // 2. If x is +0𝔽 and y is -0𝔽, return false.
    // 3. If x is -0𝔽 and y is +0𝔽, return false.
    // 4. If x is the same Number value as y, return true.
    // 5. Return false.
    x == y && x.signum() == y.signum()
  }

  /// https://tc39.es/ecma262/#sec-numeric-types-number-sameValueZero
  pub fn same_value_zero(x: &f64, y: &f64) -> bool {
    // 1. If x is NaN and y is NaN, return true.
    if x.is_nan() && y.is_nan() {
      return true;
    }
    // 2. If x is +0𝔽 and y is -0𝔽, return true.
    // 3. If x is -0𝔽 and y is +0𝔽, return true.
    // 4. If x is the same Number value as y, return true.
    // 5. Return false.
    x == y
  }

  /// https://tc39.es/ecma262/#sec-numeric-types-number-tostring
  pub fn to_string(x: &f64) -> JsString {
    if x.is_nan() {
      return "NaN".to_owned();
    }
    if x == &0.0 {
      return "0".to_owned();
    }
    if x < &0.0 {
      return format!("-{}", JsNumber::to_string(&-x));
    }
    if x.is_infinite() {
      return "Infinity".to_owned();
    }
    format!("{}", x)
  }
}
