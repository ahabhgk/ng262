use crate::language_types::{object::JsObject, Value};

#[derive(Debug, Clone)]
pub enum GetSet {
  Object(JsObject),
  Undefined,
}

impl Default for GetSet {
  fn default() -> Self {
    Self::Undefined
  }
}

impl From<GetSet> for Value {
  fn from(get_set: GetSet) -> Self {
    match get_set {
      GetSet::Undefined => Self::Undefined,
      GetSet::Object(o) => Self::Object(o),
    }
  }
}

/// https://tc39.es/ecma262/#sec-property-descriptor-specification-type
#[derive(Debug, Default)]
pub struct PropertyDescriptor {
  pub value: Option<Value>,
  pub writable: Option<bool>,
  pub get: Option<GetSet>,
  pub set: Option<GetSet>,
  pub enumerable: Option<bool>,
  pub configurable: Option<bool>,
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

  pub fn value(mut self, v: Value) -> Self {
    self.value = Some(v);
    self
  }

  pub fn writable(mut self, v: bool) -> Self {
    self.writable = Some(v);
    self
  }

  pub fn get(mut self, v: GetSet) -> Self {
    self.get = Some(v);
    self
  }

  pub fn set(mut self, v: GetSet) -> Self {
    self.set = Some(v);
    self
  }

  pub fn enumerable(mut self, v: bool) -> Self {
    self.enumerable = Some(v);
    self
  }

  pub fn configurable(mut self, v: bool) -> Self {
    self.configurable = Some(v);
    self
  }

  pub fn is_empty(&self) -> bool {
    self.value.is_none()
      && self.writable.is_none()
      && self.get.is_none()
      && self.set.is_none()
      && self.enumerable.is_none()
      && self.configurable.is_none()
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
