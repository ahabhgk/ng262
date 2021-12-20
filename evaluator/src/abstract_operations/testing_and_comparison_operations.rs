//! https://tc39.es/ecma262/#sec-testing-and-comparison-operations

use crate::language_types::{
  big_int::JsBigInt, boolean::JsBoolean, number::JsNumber, object::JsObject,
  Value,
};

impl Value {
  /// https://tc39.es/ecma262/#sec-iscallable
  pub fn is_callable(&self) -> bool {
    // 1. If Type(argument) is not Object, return false.
    match self {
      Self::Object(v) => {
        // 2. If argument has a [[Call]] internal method, return true.
        if v.get_call().is_some() {
          return true;
        }
        // 3. Return false.
        false
      }
      _ => false,
    }
  }

  /// https://tc39.es/ecma262/#sec-ispropertykey
  pub fn is_property_key(&self) -> bool {
    // 1. If Type(argument) is String, return true.
    // 2. If Type(argument) is Symbol, return true.
    // 3. Return false.
    matches!(self, Self::String(_) | Self::Symbol(_))
  }
}

/// https://tc39.es/ecma262/#sec-samevalue
pub fn same_value(x: &Value, y: &Value) -> JsBoolean {
  // 1. If Type(x) is different from Type(y), return false.
  match (x, y) {
    // 2. If Type(x) is Number, then
    //   a. Return ! Number::sameValue(x, y).
    (Value::Number(x), Value::Number(y)) => JsNumber::same_value(x, y),
    // 3. If Type(x) is BigInt, then
    //   a. Return ! BigInt::sameValue(x, y).
    (Value::BigInt(x), Value::BigInt(y)) => JsBigInt::same_value(x, y),
    // 4. Return ! SameValueNonNumeric(x, y).
    _ if matches!(
      (x, y),
      (Value::Boolean(_), Value::Boolean(_))
        | (Value::Null(_), Value::Null(_))
        | (Value::Undefined(_), Value::Undefined(_))
        | (Value::String(_), Value::String(_))
        | (Value::Object(_), Value::Object(_))
        | (Value::Symbol(_), Value::Symbol(_))
    ) =>
    {
      same_value_non_numeric(x, y)
    }
    _ => JsBoolean::False,
  }
}

/// https://tc39.es/ecma262/#sec-samevaluenonnumeric
pub fn same_value_non_numeric(x: &Value, y: &Value) -> JsBoolean {
  // 1. Assert: Type(x) is the same as Type(y).
  match (x, y) {
    // 2. If Type(x) is Undefined, return true.
    (Value::Undefined(_), Value::Undefined(_)) => JsBoolean::True,
    // 3. If Type(x) is Null, return true.
    (Value::Null(_), Value::Null(_)) => JsBoolean::True,
    // 4. If Type(x) is String, then
    //   a. If x and y are exactly the same sequence of code units (same length and same code units at corresponding indices), return true; otherwise, return false.
    (Value::String(x), Value::String(y)) => (x == y).into(),
    // 5. If Type(x) is Boolean, then
    //   a. If x and y are both true or both false, return true; otherwise, return false.
    (Value::Boolean(x), Value::Boolean(y)) => (x == y).into(),
    // 6. If Type(x) is Symbol, then
    //   a. If x and y are both the same Symbol value, return true; otherwise, return false.
    (Value::Symbol(x), Value::Symbol(y)) => (x == y).into(),
    // 7. If x and y are the same Object value, return true. Otherwise, return false.
    (Value::Object(x), Value::Object(y)) => JsObject::equals(x, y).into(),

    (Value::Number(_), Value::Number(_))
    | (Value::BigInt(_), Value::BigInt(_)) => panic!("expect non numeric type"),
    _ => panic!("expect same type"),
  }
}
