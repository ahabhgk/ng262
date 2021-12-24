use crate::{
  language_types::{
    object::{JsObject, PropertyKey},
    Value,
  },
  specification_types::property_descriptor::PropertyDescriptor,
};

/// https://tc39.es/ecma262/#sec-createdataproperty
pub fn create_data_property(
  o: &JsObject,
  p: PropertyKey,
  v: Value,
) -> Result<bool, Value> {
  // 1. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true }.
  let new_desc = PropertyDescriptor::empty()
    .value(v)
    .writable(true)
    .enumerable(true)
    .configurable(true);
  // 2. Return ? O.[[DefineOwnProperty]](P, newDesc).
  o.define_own_property(p, new_desc)
}

/// https://tc39.es/ecma262/#sec-call
pub fn call(
  f: &Value,
  v: &Value,
  arguments_list: &[Value],
) -> Result<Value, Value> {
  // 1. If argumentsList is not present, set argumentsList to a new empty List.
  // 2. If IsCallable(F) is false, throw a TypeError exception.
  if !f.is_callable() {
    return Err(Value::Undefined); // TODO: TypeError
  }
  // 3. Return ? F.[[Call]](V, argumentsList).
  f.as_object().unwrap().call(v, arguments_list)
}
