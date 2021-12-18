//! https://tc39.es/ecma262/#sec-ecmascript-language-types

pub mod big_int;
pub mod boolean;
pub mod null;
pub mod number;
pub mod object;
pub mod string;
pub mod symbol;
pub mod undefined;

use self::{
  big_int::JsBigInt, boolean::JsBoolean, null::JsNull, number::JsNumber,
  object::JsObject, string::JsString, symbol::JsSymbol, undefined::JsUndefined,
};

pub enum Value {
  Undefined(JsUndefined),
  Null(JsNull),
  Boolean(JsBoolean),
  String(JsString),
  Symbol(JsSymbol),
  Number(JsNumber),
  BigInt(JsBigInt),
  Object(JsObject),
}
