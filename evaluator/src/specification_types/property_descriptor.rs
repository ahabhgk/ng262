use crate::language_types::Value;

/// https://tc39.es/ecma262/#sec-property-descriptor-specification-type
pub struct PropertyDescriptor {
  value: Option<Value>,
  writable: Option<bool>,
  get: Option<Value>, // enum Get { JsObject, Undefined } ?
  set: Option<Value>, // enum Set { JsObject, Undefined } ?
  enumerable: Option<bool>,
  configurable: Option<bool>,
}

impl PropertyDescriptor {
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
