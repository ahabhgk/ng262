use std::ops::Deref;

use num_bigint::BigInt;

use super::boolean::JsBoolean;

/// https://tc39.es/ecma262/#sec-ecmascript-language-types-bigint-type
pub struct JsBigInt(BigInt);

impl Deref for JsBigInt {
  type Target = BigInt;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl JsBigInt {
  /// https://tc39.es/ecma262/#sec-numeric-types-bigint-equal
  pub fn equal(x: &Self, y: &Self) -> JsBoolean {
    // 1. If ℝ(x) = ℝ(y), return true; otherwise return false.
    (**x == **y).into()
  }

  /// https://tc39.es/ecma262/#sec-numeric-types-bigint-sameValue
  pub fn same_value(x: &Self, y: &Self) -> JsBoolean {
    // 1. Return BigInt::equal(x, y).
    Self::equal(x, y)
  }
}
