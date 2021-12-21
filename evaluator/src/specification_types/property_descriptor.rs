use crate::language_types::{boolean::JsBoolean, object::JsObject, Value};

#[derive(Debug, Clone)]
pub enum Get {
  Object(JsObject),
  Undefined,
}

#[derive(Debug, Clone)]
pub enum Set {
  Object(JsObject),
  Undefined,
}

/// https://tc39.es/ecma262/#sec-property-descriptor-specification-type
#[derive(Debug)]
pub struct PropertyDescriptor {
  pub value: Option<Value>,
  pub writable: Option<JsBoolean>,
  pub get: Option<Get>,
  pub set: Option<Set>,
  pub enumerable: Option<JsBoolean>,
  pub configurable: Option<JsBoolean>,
}

impl Default for PropertyDescriptor {
  fn default() -> Self {
    Self {
      value: Some(Value::Undefined),
      writable: Some(JsBoolean::False),
      get: Some(Get::Undefined),
      set: Some(Set::Undefined),
      enumerable: Some(JsBoolean::False),
      configurable: Some(JsBoolean::False),
    }
  }
}

impl PropertyDescriptor {
  pub fn empty() -> Self {
    Self {
      value: None,
      writable: None,
      get: None,
      set: None,
      enumerable: None,
      configurable: None,
    }
  }

  /// https://tc39.es/ecma262/#sec-isaccessordescriptor
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
