use std::ops::Deref;

use super::boolean::JsBoolean;

/// https://tc39.es/ecma262/#sec-ecmascript-language-types-number-type
#[derive(Debug, Clone, Copy)]
pub struct JsNumber(f64);

impl Deref for JsNumber {
  type Target = f64;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl JsNumber {
  /// https://tc39.es/ecma262/#sec-numeric-types-number-sameValue
  pub fn same_value(x: &Self, y: &Self) -> JsBoolean {
    // 1. If x is NaN and y is NaN, return true.
    if x.is_nan() && y.is_nan() {
      return JsBoolean::True;
    }
    // 2. If x is +0𝔽 and y is -0𝔽, return false.
    // 3. If x is -0𝔽 and y is +0𝔽, return false.
    // 4. If x is the same Number value as y, return true.
    // 5. Return false.
    if **x == **y && x.signum() == y.signum() {
      JsBoolean::True
    } else {
      JsBoolean::False
    }
  }
}
