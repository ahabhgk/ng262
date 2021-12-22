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

  pub fn get_properties_mut(&self) -> RefMut<'_, PropertyMap> {
    RefMut::map(self.0.borrow_mut(), |o| &mut o.properties)
  }
}

impl JsObject {
  /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
  pub fn get_prototype_of(&self) -> Result<Prototype, Value> {
    let f = self.get_internal_methods().get_prototype_of;
    f(self)
  }

  /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
  pub fn set_prototype_of(&self, v: Prototype) -> Result<bool, Value> {
    let f = self.get_internal_methods().set_prototype_of;
    f(self, v)
  }

  /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible
  pub fn is_extensible(&self) -> Result<bool, Value> {
    let f = self.get_internal_methods().is_extensible;
    f(self)
  }

  /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
  pub fn prevent_extensions(&self) -> Result<bool, Value> {
    let f = self.get_internal_methods().prevent_extensions;
    f(self)
  }

  /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
  pub fn get_own_property(
    &self,
    p: &PropertyKey,
  ) -> Result<Option<PropertyDescriptor>, Value> {
    let f = self.get_internal_methods().get_own_property;
    f(self, p)
  }

  /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc
  pub fn define_own_property(
    &self,
    p: PropertyKey,
    desc: PropertyDescriptor,
  ) -> Result<bool, Value> {
    let f = self.get_internal_methods().define_own_property;
    f(self, p, desc)
  }

  /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
  pub fn has_property(&self, p: &PropertyKey) -> Result<bool, Value> {
    let f = self.get_internal_methods().has_property;
    f(self, p)
  }

  /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver
  pub fn get(&self, p: &PropertyKey, receiver: &Value) -> Result<Value, Value> {
    let f = self.get_internal_methods().get;
    f(self, p, receiver)
  }

  /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver
  pub fn set(
    &self,
    p: PropertyKey,
    v: Value,
    receiver: &Value,
  ) -> Result<Value, Value> {
    let f = self.get_internal_methods().set;
    f(self, p, v, receiver)
  }
}

impl JsObject {
  pub fn call(
    &self,
    this_argument: &Value,
    arguments_list: &[Value],
  ) -> Result<Value, Value> {
    let f = self.get_internal_methods().call;
    f.expect(
      "called `[[Call]]` for object without a `[[Call]]` internal method",
    )(self, this_argument, arguments_list)
  }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PropertyKey {
  String(JsString),
  Symbol(JsSymbol),
}

pub struct InternalMethods {
  pub get_prototype_of: fn(&JsObject) -> Result<Prototype, Value>,
  pub set_prototype_of: fn(&JsObject, Prototype) -> Result<bool, Value>,
  pub is_extensible: fn(&JsObject) -> Result<bool, Value>,
  pub prevent_extensions: fn(&JsObject) -> Result<bool, Value>,
  pub get_own_property:
    fn(&JsObject, &PropertyKey) -> Result<Option<PropertyDescriptor>, Value>,
  pub define_own_property:
    fn(&JsObject, PropertyKey, PropertyDescriptor) -> Result<bool, Value>,
  pub has_property: fn(&JsObject, &PropertyKey) -> Result<bool, Value>,
  pub get: fn(&JsObject, &PropertyKey, &Value) -> Result<Value, Value>,
  pub set: fn(&JsObject, PropertyKey, Value, &Value) -> Result<Value, Value>,

  pub call: Option<fn(&JsObject, &Value, &[Value]) -> Result<Value, Value>>,
}

impl fmt::Debug for InternalMethods {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "internal methods")
  }
}
