//! https://tc39.es/ecma262/#sec-ecmascript-language-types

pub mod big_int;
pub mod boolean;
pub mod number;
pub mod object;
pub mod string;
pub mod symbol;

use self::{
  big_int::JsBigInt, boolean::JsBoolean, number::JsNumber, object::JsObject,
  string::JsString, symbol::JsSymbol,
};

#[derive(Debug, Clone)]
pub enum Value {
  /// https://tc39.es/ecma262/#sec-ecmascript-language-types-undefined-type
  Undefined,
  /// https://tc39.es/ecma262/#sec-ecmascript-language-types-null-type
  Null,
  /// https://tc39.es/ecma262/#sec-ecmascript-language-types-boolean-type
  Boolean(bool),
  /// https://tc39.es/ecma262/#sec-ecmascript-language-types-string-type
  String(JsString),
  /// https://tc39.es/ecma262/#sec-ecmascript-language-types-symbol-type
  Symbol(JsSymbol),
  /// https://tc39.es/ecma262/#sec-ecmascript-language-types-number-type
  Number(f64),
  /// https://tc39.es/ecma262/#sec-ecmascript-language-types-bigint-type
  BigInt(JsBigInt),
  /// https://tc39.es/ecma262/#sec-object-type
  Object(JsObject),
}

impl Default for Value {
  fn default() -> Self {
    Self::Undefined
  }
}

impl Value {
  pub fn as_object(&self) -> Option<&JsObject> {
    match self {
      Self::Object(o) => Some(o),
      _ => None,
    }
  }
}
