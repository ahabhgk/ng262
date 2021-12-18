use crate::language_types::{string::JsString, Value};

/// https://tc39.es/ecma262/#sec-completion-record-specification-type
pub struct Completion {
  r#type: Type,
  value: Option<Value>,
  target: Option<JsString>,
}

impl Completion {
  /// https://tc39.es/ecma262/#sec-throwcompletion
  pub fn throw(value: Value) -> Self {
    Self {
      r#type: Type::Throw,
      value: Some(value),
      target: None,
    }
  }

  /// https://tc39.es/ecma262/#sec-normalcompletion
  pub fn normal(value: Value) -> Self {
    Self {
      r#type: Type::Normal,
      value: Some(value),
      target: None,
    }
  }
}

pub enum Type {
  Normal,
  Break,
  Continue,
  Return,
  Throw,
}
