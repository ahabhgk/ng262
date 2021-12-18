use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
  helpers::Either, specification_types::property_descriptor::PropertyDescriptor,
};

use super::{null::JsNull, string::JsString, symbol::JsSymbol, Value};

pub type Prototype = Either<JsObject, JsNull>;

struct Inner {
  properties: PropertyMap,
  pub internal_methods: &'static InternalMethods,
  prototype: Prototype,
  extensible: bool,
}

impl Inner {
  pub fn call(&self, v: &Value, arguments_list: &[Value]) {
    let call = self.internal_methods.call;
  }
}

/// https://tc39.es/ecma262/#sec-object-type
#[derive(Clone)]
pub struct JsObject(Rc<RefCell<Inner>>);

impl AsRef<RefCell<Inner>> for JsObject {
  fn as_ref(&self) -> &RefCell<Inner> {
    &*self.0
  }
}

impl JsObject {
  pub fn get_call(&self) -> Option<fn(&JsObject, &[Value]) -> Value> {
    self.0.borrow().internal_methods.call
  }

  pub fn get_prototype(&self) -> Prototype {
    self.0.borrow().prototype.clone()
  }

  pub fn equals(lhs: &Self, rhs: &Self) -> bool {
    std::ptr::eq(lhs.as_ref(), rhs.as_ref())
  }
}

pub struct PropertyMap {
  /// Properties
  string_properties: HashMap<JsString, PropertyDescriptor>,
  /// Symbol Properties
  symbol_properties: HashMap<JsSymbol, PropertyDescriptor>,
}

pub struct InternalMethods {
  get_prototype_of: fn(&JsObject) -> Prototype, // TODO
  call: Option<fn(&JsObject, &[Value]) -> Value>, // TODO
}
