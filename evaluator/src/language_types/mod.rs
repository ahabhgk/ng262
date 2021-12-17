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

pub enum Value {
  /// https://tc39.es/ecma262/#sec-ecmascript-language-types-undefined-type
  Undefined,
  /// https://tc39.es/ecma262/#sec-ecmascript-language-types-null-type
  Null,
  Boolean(JsBoolean),
  String(JsString),
  Symbol(JsSymbol),
  Number(JsNumber),
  BigInt(JsBigInt),
  Object(JsObject),
}
