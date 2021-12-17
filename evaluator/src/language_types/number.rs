use std::ops::Deref;

/// https://tc39.es/ecma262/#sec-ecmascript-language-types-number-type
#[derive(Debug, Clone, Copy)]
pub struct JsNumber(f64);

impl Deref for JsNumber {
  type Target = f64;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
