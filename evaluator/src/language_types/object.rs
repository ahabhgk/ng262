use std::collections::HashMap;

use crate::specification_types::property_descriptor::PropertyDescriptor;

use super::{string::JsString, symbol::JsSymbol};

/// https://tc39.es/ecma262/#sec-object-type
pub struct JsObject {
  properties: PropertyMap,
}

pub struct PropertyMap {
  /// Properties
  string_properties: HashMap<JsString, PropertyDescriptor>,
  /// Symbol Properties
  symbol_properties: HashMap<JsSymbol, PropertyDescriptor>,
}
