use std::{
  cell::{Ref, RefCell, RefMut},
  collections::HashMap,
  fmt,
  rc::Rc,
};

use crate::specification_types::property_descriptor::PropertyDescriptor;

use super::{string::JsString, symbol::JsSymbol, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prototype {
  Object(JsObject),
  Null,
}

pub type PropertyMap = HashMap<PropertyKey, PropertyDescriptor>;

#[derive(Debug)]
struct Inner {
  properties: PropertyMap,
  internal_methods: &'static InternalMethods,
  prototype: Prototype,
  extensible: bool,
}

#[derive(Debug, Clone)]
pub struct JsObject(Rc<RefCell<Inner>>);

impl PartialEq for JsObject {
  fn eq(&self, other: &Self) -> bool {
    let a: &RefCell<Inner> = self.as_ref();
    std::ptr::eq(a, other.as_ref())
  }
}

impl Eq for JsObject {}

impl AsRef<RefCell<Inner>> for JsObject {
  fn as_ref(&self) -> &RefCell<Inner> {
    &*self.0
  }
}

impl JsObject {
  pub fn call(&self) -> Option<fn(&JsObject, &[Value]) -> Value> {
    self.0.borrow().internal_methods.call
  }

  pub fn prototype(&self) -> Ref<'_, Prototype> {
    Ref::map(self.0.borrow(), |o| &o.prototype)
  }

  pub fn set_prototype(&self, p: Prototype) {
    self.0.borrow_mut().prototype = p;
  }

  pub fn extensible(&self) -> bool {
    self.0.borrow().extensible
  }

  pub fn set_extensible(&self, e: bool) {
    self.0.borrow_mut().extensible = e;
  }

  pub fn get_internal_methods(&self) -> &InternalMethods {
    self.0.borrow().internal_methods
  }

  pub fn get_properties(&self) -> Ref<'_, PropertyMap> {
    Ref::map(self.0.borrow(), |o| &o.properties)
  }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PropertyKey {
  String(JsString),
  Symbol(JsSymbol),
}

pub struct InternalMethods {
  pub get_prototype_of: fn(&JsObject) -> Prototype, // TODO
  pub call: Option<fn(&JsObject, &[Value]) -> Value>, // TODO
}

impl fmt::Debug for InternalMethods {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "internal methods")
  }
}
