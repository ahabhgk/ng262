//! https://tc39.es/ecma262/#sec-type-conversion

use num_traits::Zero;

use crate::language_types::{boolean::JsBoolean, Value};

impl Value {
  /// https://tc39.es/ecma262/#sec-toboolean
  pub fn to_boolean(&self) -> bool {
    match self {
      Value::Undefined => false,
      Value::Null => false,
      Value::Symbol(_) => true,
      Value::Object(_) => true,
      Value::Boolean(v) => *v,
      Value::String(s) => {
        if s.is_empty() {
          false
        } else {
          true
        }
      }
      Value::Number(n) => {
        if *n == 0.0 || n.is_nan() {
          false
        } else {
          true
        }
      }
      Value::BigInt(v) => {
        if v.is_zero() {
          false
        } else {
          true
        }
      }
    }
  }
}
