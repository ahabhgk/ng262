use std::ops::Deref;

use num_bigint::BigInt;

use super::{boolean::JsBoolean, Value};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct JsBigInt(BigInt);

impl Deref for JsBigInt {
  type Target = BigInt;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl JsBigInt {
  /// https://tc39.es/ecma262/#sec-numeric-types-bigint-equal
  pub fn equal(x: &Self, y: &Self) -> bool {
    // 1. If ℝ(x) = ℝ(y), return true; otherwise return false.
    x == y
  }

  /// https://tc39.es/ecma262/#sec-numeric-types-bigint-sameValue
  pub fn same_value(x: &Self, y: &Self) -> bool {
    // 1. Return BigInt::equal(x, y).
    Self::equal(x, y)
  }

  /// https://tc39.es/ecma262/#sec-numeric-types-bigint-sameValueZero
  pub fn same_value_zero(x: &Self, y: &Self) -> bool {
    // 1. Return BigInt::equal(x, y).
    Self::equal(x, y)
  }
}
