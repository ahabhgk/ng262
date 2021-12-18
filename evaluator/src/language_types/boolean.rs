/// https://tc39.es/ecma262/#sec-ecmascript-language-types-boolean-type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsBoolean {
  True,
  False,
}

impl From<bool> for JsBoolean {
  fn from(b: bool) -> Self {
    if b {
      Self::True
    } else {
      Self::False
    }
  }
}
