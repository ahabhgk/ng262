use std::ops::Deref;

use num_bigint::BigInt;
use num_traits::{Signed, Zero};

use super::string::JsString;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct JsBigInt(BigInt);

impl Deref for JsBigInt {
  type Target = BigInt;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl JsBigInt {
  /// https://tc39.es/ecma262/#sec-numeric-types-bigint-unaryMinus
  pub fn unary_minus(x: &Self) -> Self {
    // 1. If x is 0ℤ, return 0ℤ.
    if x.is_zero() {
      return Self(BigInt::zero());
    }
    // 2. Return the BigInt value that represents the negation of ℝ(x).
    Self(&BigInt::zero() - &**x)
  }

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

  /// https://tc39.es/ecma262/#sec-numeric-types-bigint-tostring
  pub fn to_string(x: &Self) -> JsString {
    // 1. If x < 0ℤ, return the string-concatenation of the String "-" and ! BigInt::toString(-x).
    if x.is_negative() {
      return Self::to_string(&Self::unary_minus(x));
    }
    // 2. Return the String value consisting of the code units of the digits of the decimal representation of x.
    x.to_str_radix(10)
  }
}
