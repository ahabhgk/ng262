//! https://tc39.es/ecma262/#sec-type-conversion

use num_traits::Zero;

use crate::language_types::{string::{string_to_number, JsString}, Value, number::JsNumber, big_int::JsBigInt};

impl Value {
  /// https://tc39.es/ecma262/#sec-toprimitive
  pub fn to_primitive(&self) -> Result<Value, Value> {
    todo!()
  }

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

  /// https://tc39.es/ecma262/#sec-tonumber
  pub fn to_number(&self) -> Result<f64, Value> {
    match self {
      Self::Undefined => Ok(f64::NAN),
      Self::Null => Ok(0.0),
      Self::Boolean(argument) => {
        if *argument {
          Ok(1.0)
        } else {
          Ok(0.0)
        }
      }
      Self::Number(argument) => Ok(*argument),
      Self::String(argument) => Ok(string_to_number(argument)),
      Self::Symbol(_) | Self::BigInt(_) => Err(Value::Undefined), // TODO: TypeError
      argument @ Self::Object(_) => {
        let prim_value = argument.to_primitive()?;
        prim_value.to_number()
      }
    }
  }

  /// https://tc39.es/ecma262/#sec-tostring
  pub fn to_string(&self) -> Result<JsString, Value> {
    match self {
      Value::Undefined => Ok("undefined".to_owned()),
      Value::Null => Ok("null".to_owned()),
      Value::Boolean(argument) => {
        if *argument {
          Ok("true".to_owned())
        } else {
          Ok("false".to_owned())
        }
      }
      Value::Number(argument) => Ok(JsNumber::to_string(argument)),
      Value::String(argument) => Ok(argument.to_owned()),
      Value::Symbol(_) => Err(Value::Undefined), // TODO: TypeError
      Value::BigInt(argument) => Ok(JsBigInt::to_string(argument)),
      argument @ Value::Object(_) => {
        let prim_value = argument.to_primitive()?;
        prim_value.to_string()
      }
    }
  }

  /// https://tc39.es/ecma262/#sec-canonicalnumericindexstring
  pub fn canonical_numeric_index_string(&self) -> Result<Option<f64>, Value> {
    match self {
      // 1. If argument is "-0", return -0𝔽.
      Self::String(argument) if argument == "-0" => return Ok(Some(-0.0)),
      argument @ Value::String(s) => {
        // 2. Let n be ! ToNumber(argument).
        let n = argument.to_number()?;
        // 3. If SameValue(! ToString(n), argument) is false, return undefined.
        if &Value::Number(n).to_string()? != s {
          return Ok(None);
        }
        // 4. Return n.
        return Ok(Some(n));
      }
      _ => panic!("should be a string"),
    }
  }
}
