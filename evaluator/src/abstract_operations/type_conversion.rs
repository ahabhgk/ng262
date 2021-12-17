use num_traits::Zero;

use crate::language_types::{boolean::JsBoolean, Value};

impl Value {
  /// https://tc39.es/ecma262/#sec-toboolean
  pub fn to_boolean(&self) -> JsBoolean {
    match self {
      Value::Undefined => JsBoolean::False,
      Value::Null => JsBoolean::False,
      Value::Symbol(_) => JsBoolean::True,
      Value::Object(_) => JsBoolean::True,
      Value::Boolean(v) => *v,
      Value::String(s) => {
        if s.is_empty() {
          JsBoolean::False
        } else {
          JsBoolean::True
        }
      }
      Value::Number(n) => {
        if **n == 0.0 || n.is_nan() {
          JsBoolean::False
        } else {
          JsBoolean::True
        }
      }
      Value::BigInt(v) => {
        if v.is_zero() {
          JsBoolean::False
        } else {
          JsBoolean::True
        }
      }
    }
  }
}
