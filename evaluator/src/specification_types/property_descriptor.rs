use crate::{
  helpers::Either,
  language_types::{
    boolean::JsBoolean, object::JsObject, undefined::JsUndefined, Value,
  },
};

/// https://tc39.es/ecma262/#sec-property-descriptor-specification-type
pub struct PropertyDescriptor {
  value: Option<Value>,
  writable: Option<JsBoolean>,
  get: Option<Either<JsObject, JsUndefined>>,
  set: Option<Either<JsObject, JsUndefined>>,
  enumerable: Option<JsBoolean>,
  configurable: Option<JsBoolean>,
}

impl Default for PropertyDescriptor {
  fn default() -> Self {
    Self {
      value: Some(Value::Undefined(JsUndefined)),
      writable: Some(JsBoolean::False),
      get: Some(Either::B(JsUndefined)),
      set: Some(Either::B(JsUndefined)),
      enumerable: Some(JsBoolean::False),
      configurable: Some(JsBoolean::False),
    }
  }
}

/// https://tc39.es/ecma262/#sec-isaccessordescriptor
impl PropertyDescriptor {
  pub fn is_accessor_descriptor(&self) -> bool {
    // 1. If Desc is undefined, return false.
    // 2. If both Desc.[[Get]] and Desc.[[Set]] are absent, return false.
    if self.get.is_none() && self.set.is_none() {
      return false;
    }
    // 3. Return true.
    true
  }

  /// https://tc39.es/ecma262/#sec-isdatadescriptor
  pub fn is_data_descriptor(&self) -> bool {
    // 1. If Desc is undefined, return false.
    // 2. If both Desc.[[Value]] and Desc.[[Writable]] are absent, return false.
    if self.value.is_none() && self.writable.is_none() {
      return false;
    }
    // 3. Return true.
    true
  }

  /// https://tc39.es/ecma262/#sec-isgenericdescriptor
  pub fn is_generic_descriptor(&self) -> bool {
    // 1. If Desc is undefined, return false.
    // 2. If IsAccessorDescriptor(Desc) and IsDataDescriptor(Desc) are both false, return true.
    if !self.is_accessor_descriptor() && !self.is_data_descriptor() {
      return true;
    }
    // 3. Return false.
    false
  }
}
