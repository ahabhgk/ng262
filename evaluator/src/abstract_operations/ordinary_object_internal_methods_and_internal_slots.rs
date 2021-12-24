use crate::{
  abstract_operations::operations_on_objects::{call, create_data_property},
  language_types::{
    object::{JsObject, PropertyKey, Prototype},
    Value,
  },
  specification_types::property_descriptor::{GetSet, PropertyDescriptor},
};

use super::{
  data_types_and_values::is_array_index,
  testing_and_comparison_operations::{is_extensible, same_value},
};

/// https://tc39.es/ecma262/#sec-ordinarygetprototypeof
pub fn ordinary_get_prototype_of(o: &JsObject) -> Result<Prototype, Value> {
  // 1. Return O.[[Prototype]].
  Ok(o.prototype().clone())
}

/// https://tc39.es/ecma262/#sec-ordinarysetprototypeof
pub fn ordinary_set_prototype_of(
  o: &JsObject,
  v: Prototype,
) -> Result<bool, Value> {
  // 1. Let current be O.[[Prototype]].
  let current = o.prototype().clone();
  // 2. If SameValue(V, current) is true, return true.
  if v == current {
    return Ok(true);
  }
  // 3. Let extensible be O.[[Extensible]].
  let extensible = o.extensible();
  // 4. If extensible is false, return false.
  if extensible.into() {
    return Ok(false);
  }
  // 5. Let p be V.
  let mut p = v.clone();
  // 6. Let done be false.
  // 7. Repeat, while done is false,
  // a. If p is null, set done to true.
  while let Prototype::Object(proto) = p {
    // b. Else if SameValue(p, O) is true, return false.
    if proto == *o {
      return Ok(false);
      // c. Else,
    } else {
      // i. If p.[[GetPrototypeOf]] is not the ordinary object internal method defined in 10.1.1, set done to true.
      if proto.get_internal_methods().get_prototype_of as usize
        != ordinary_get_prototype_of as usize
      {
        break;
        // ii. Else, set p to p.[[Prototype]].
      } else {
        p = proto.prototype().clone();
      }
    }
  }
  // 8. Set O.[[Prototype]] to V.
  o.set_prototype(v);
  // 9. Return true.
  Ok(true)
}

/// https://tc39.es/ecma262/#sec-ordinaryisextensible
pub fn ordinary_is_extensible(o: &JsObject) -> Result<bool, Value> {
  // 1. Return O.[[Extensible]].
  Ok(o.extensible())
}

/// https://tc39.es/ecma262/#sec-ordinarypreventextensions
pub fn ordinary_prevent_extensions(o: &JsObject) -> Result<bool, Value> {
  // 1. Set O.[[Extensible]] to false.
  o.set_extensible(false);
  // 2. Return true.
  Ok(true)
}

/// https://tc39.es/ecma262/#sec-ordinarygetownproperty
pub fn ordinary_get_own_property(
  o: &JsObject,
  p: &PropertyKey,
) -> Result<Option<PropertyDescriptor>, Value> {
  // 1. If O does not have an own property with key P, return undefined.
  match o.get_properties().get(p) {
    None => Ok(None),
    Some(x) => {
      // 2. Let D be a newly created Property Descriptor with no fields.
      // 3. Let X be O's own property whose key is P.
      let mut d = PropertyDescriptor::empty();
      // 4. If X is a data property, then
      if x.is_data_descriptor() {
        // a. Set D.[[Value]] to the value of X's [[Value]] attribute.
        d.value = x.value.clone();
        // b. Set D.[[Writable]] to the value of X's [[Writable]] attribute.
        d.writable = x.writable;
        // 5. Else,
        // a. Assert: X is an accessor property.
      } else if x.is_accessor_descriptor() {
        // b. Set D.[[Get]] to the value of X's [[Get]] attribute.
        d.get = x.get.clone();
        // c. Set D.[[Set]] to the value of X's [[Set]] attribute.
        d.set = x.set.clone();
      }
      // 6. Set D.[[Enumerable]] to the value of X's [[Enumerable]] attribute.
      d.enumerable = x.enumerable;
      // 7. Set D.[[Configurable]] to the value of X's [[Configurable]] attribute.
      d.configurable = x.configurable;
      // 8. Return D.
      Ok(Some(d))
    }
  }
}

/// https://tc39.es/ecma262/#sec-ordinarydefineownproperty
pub fn ordinary_define_own_property(
  o: &JsObject,
  p: PropertyKey,
  desc: PropertyDescriptor,
) -> Result<bool, Value> {
  // 1. Let current be ? O.[[GetOwnProperty]](P).
  let current = o.get_own_property(&p)?;
  // 2. Let extensible be ? IsExtensible(O).
  let extensible = is_extensible(o)?;
  // 3. Return ValidateAndApplyPropertyDescriptor(O, P, extensible, Desc, current).
  validate_and_apply_property_descriptor(
    Some((o, p)),
    extensible,
    desc,
    current,
  )
}

/// https://tc39.es/ecma262/#sec-iscompatiblepropertydescriptor
pub fn is_compatible_property_descriptor(
  extensible: bool,
  desc: PropertyDescriptor,
  current: Option<PropertyDescriptor>,
) -> Result<bool, Value> {
  // 1. Return ValidateAndApplyPropertyDescriptor(undefined, undefined, Extensible, Desc, Current).
  validate_and_apply_property_descriptor(None, extensible, desc, current)
}

/// https://tc39.es/ecma262/#sec-validateandapplypropertydescriptor
pub fn validate_and_apply_property_descriptor(
  o_and_p: Option<(&JsObject, PropertyKey)>,
  extensible: bool,
  desc: PropertyDescriptor,
  current: Option<PropertyDescriptor>,
) -> Result<bool, Value> {
  // 1. Assert: If O is not undefined, then IsPropertyKey(P) is true.
  // 2. If current is undefined, then
  match current {
    None => {
      // a. If extensible is false, return false.
      if !extensible {
        return Ok(false);
      }
      // b. Assert: extensible is true.
      assert!(extensible);
      // c. If IsGenericDescriptor(Desc) is true or IsDataDescriptor(Desc) is true, then
      if desc.is_generic_descriptor() || desc.is_data_descriptor() {
        // i. If O is not undefined, create an own data property named P of object O
        // whose [[Value]], [[Writable]], [[Enumerable]], and [[Configurable]] attribute
        // values are described by Desc. If the value of an attribute field of Desc is
        // absent, the attribute of the newly created property is set to its default value.
        if let Some((o, p)) = o_and_p {
          o.get_properties_mut().insert(
            p,
            PropertyDescriptor {
              value: Some(desc.value.unwrap_or_default()),
              writable: Some(desc.writable.unwrap_or_default()),
              enumerable: Some(desc.enumerable.unwrap_or_default()),
              configurable: Some(desc.configurable.unwrap_or_default()),
              get: None,
              set: None,
            },
          );
        }
      // d. Else,
      } else {
        // i. Assert: ! IsAccessorDescriptor(Desc) is true.
        assert!(desc.is_accessor_descriptor());
        // ii. If O is not undefined, create an own accessor property named P of object
        // O whose [[Get]], [[Set]], [[Enumerable]], and [[Configurable]] attribute values
        // are described by Desc. If the value of an attribute field of Desc is absent, the
        // attribute of the newly created property is set to its default value.
        if let Some((o, p)) = o_and_p {
          o.get_properties_mut().insert(
            p,
            PropertyDescriptor {
              get: Some(desc.get.unwrap_or_default()),
              set: Some(desc.set.unwrap_or_default()),
              enumerable: Some(desc.enumerable.unwrap_or_default()),
              configurable: Some(desc.configurable.unwrap_or_default()),
              value: None,
              writable: None,
            },
          );
        }
      }
      // e. Return true.
      return Ok(true);
    }
    Some(current) => {
      // 3. If every field in Desc is absent, return true.
      if desc.is_empty() {
        return Ok(true);
      }
      // 4. If current.[[Configurable]] is false, then
      if let Some(false) = current.configurable {
        // a. If Desc.[[Configurable]] is present and its value is true, return false.
        if let Some(true) = desc.configurable {
          return Ok(false);
        }
        // b. If Desc.[[Enumerable]] is present and ! SameValue(Desc.[[Enumerable]], current.[[Enumerable]]) is false, return false.
        if let Some(true) = desc.enumerable {
          return Ok(false);
        }
      }
      // 5. If ! IsGenericDescriptor(Desc) is true, then
      if desc.is_generic_descriptor() {
        // a. NOTE: No further validation is required.
        // 6. Else if ! SameValue(! IsDataDescriptor(current), ! IsDataDescriptor(Desc)) is false, then
      } else if current.is_data_descriptor() != desc.is_data_descriptor() {
        // a. If current.[[Configurable]] is false, return false.
        if let Some(false) = current.configurable {
          return Ok(false);
        }
        // b. If IsDataDescriptor(current) is true, then
        if current.is_data_descriptor() {
          // i. If O is not undefined, convert the property named P of object O from a
          // data property to an accessor property. Preserve the existing values of the
          // converted property's [[Configurable]] and [[Enumerable]] attributes and set
          // the rest of the property's attributes to their default values.
          if let Some((o, ref p)) = o_and_p {
            let mut property_map = o.get_properties_mut();
            let current = property_map.get_mut(p).unwrap();
            current.value = None;
            current.writable = None;
            current.get = Some(GetSet::default());
            current.set = Some(GetSet::default());
          }
        // c. Else,
        } else {
          // i. If O is not undefined, convert the property named P of object O from an
          // accessor property to a data property. Preserve the existing values of the
          // converted property's [[Configurable]] and [[Enumerable]] attributes and set
          // the rest of the property's attributes to their default values.
          if let Some((o, ref p)) = o_and_p {
            let mut property_map = o.get_properties_mut();
            let current = property_map.get_mut(p).unwrap();
            current.get = None;
            current.set = None;
            current.value = Some(Value::default());
            current.writable = Some(bool::default());
          }
        }
      // 7. Else if IsDataDescriptor(current) and IsDataDescriptor(Desc) are both true, then
      } else if current.is_data_descriptor() && desc.is_data_descriptor() {
        // a. If current.[[Configurable]] is false and current.[[Writable]] is false, then
        if matches!(current.configurable, Some(false))
          && matches!(current.writable, Some(false))
        {
          // i. If Desc.[[Writable]] is present and Desc.[[Writable]] is true, return false.
          if let Some(true) = desc.writable {
            return Ok(false);
          }
          // ii. If Desc.[[Value]] is present and SameValue(Desc.[[Value]], current.[[Value]]) is false, return false.
          if matches!(desc.value, Some(v) if !same_value(&v, &current.value.unwrap()))
          {
            return Ok(false);
          }
          // iii. Return true.
          return Ok(true);
        }
      // 8. Else,
      } else {
        // a. Assert: ! IsAccessorDescriptor(current) and ! IsAccessorDescriptor(Desc) are both true.
        assert!(
          current.is_accessor_descriptor() && desc.is_accessor_descriptor()
        );
        // b. If current.[[Configurable]] is false, then
        if let Some(false) = current.configurable {
          // i. If Desc.[[Set]] is present and SameValue(Desc.[[Set]], current.[[Set]]) is false, return false.
          if matches!(desc.set, Some(set) if !same_value(&set.clone().into(), &current.set.unwrap().into()))
          {
            return Ok(false);
          }
          // ii. If Desc.[[Get]] is present and SameValue(Desc.[[Get]], current.[[Get]]) is false, return false.
          if matches!(desc.get, Some(get) if !same_value(&get.clone().into(), &current.get.unwrap().into()))
          {
            return Ok(false);
          }
          // iii. Return true.
          return Ok(true);
        }
      }
      // 9. If O is not undefined, then
      if let Some((o, p)) = o_and_p {
        // a. For each field of Desc that is present, set the corresponding attribute of the property named P of object O to the value of the field.
        o.get_properties_mut().insert(p, current);
      }
      // 10. Return true.
      Ok(true)
    }
  }
}

/// https://tc39.es/ecma262/#sec-ordinaryhasproperty
pub fn ordinary_has_property(
  o: &JsObject,
  p: &PropertyKey,
) -> Result<bool, Value> {
  // 1. Let hasOwn be ? O.[[GetOwnProperty]](P).
  let has_own = o.get_own_property(p)?;
  // 2. If hasOwn is not undefined, return true.
  if has_own.is_some() {
    return Ok(true);
  }
  // 3. Let parent be ? O.[[GetPrototypeOf]]().
  let parent = o.get_prototype_of()?;
  // 4. If parent is not null, then
  if let Prototype::Object(parent) = parent {
    // a. Return ? parent.[[HasProperty]](P).
    return parent.has_property(p);
  }
  // 5. Return false.
  Ok(false)
}

/// https://tc39.es/ecma262/#sec-ordinaryget
pub fn ordinary_get(
  o: &JsObject,
  p: &PropertyKey,
  receiver: &Value,
) -> Result<Value, Value> {
  // 1. Let desc be ? O.[[GetOwnProperty]](P).
  let desc = o.get_own_property(p)?;
  match desc {
    // 2. If desc is undefined, then
    None => {
      // a. Let parent be ? O.[[GetPrototypeOf]]().
      let parent = o.get_prototype_of()?;
      match parent {
        // b. If parent is null, return undefined.
        Prototype::Null => return Ok(Value::Undefined),
        // c. Return ? parent.[[Get]](P, Receiver).
        Prototype::Object(parent) => return parent.get(p, receiver),
      }
    }
    Some(desc) => {
      // 3. If IsDataDescriptor(desc) is true, return desc.[[Value]].
      if desc.is_data_descriptor() {
        return Ok(desc.value.unwrap());
      }
      // 4. Assert: IsAccessorDescriptor(desc) is true.
      assert!(desc.is_accessor_descriptor());
      // 5. Let getter be desc.[[Get]].
      match desc.get.unwrap() {
        // 6. If getter is undefined, return undefined.
        GetSet::Undefined => return Ok(Value::Undefined),
        // 7. Return ? Call(getter, Receiver).
        GetSet::Object(getter) => {
          return call(&Value::Object(getter), receiver, &[])
        }
      }
    }
  }
}

/// https://tc39.es/ecma262/#sec-ordinaryset
pub fn ordinary_set(
  o: &JsObject,
  p: PropertyKey,
  v: Value,
  receiver: &Value,
) -> Result<bool, Value> {
  // 1. Let ownDesc be ? O.[[GetOwnProperty]](P).
  let own_desc = o.get_own_property(&p)?;
  // 2. Return OrdinarySetWithOwnDescriptor(O, P, V, Receiver, ownDesc).
  ordinary_set_with_own_descriptor(o, p, v, receiver, own_desc)
}

/// https://tc39.es/ecma262/#sec-ordinarysetwithowndescriptor
pub fn ordinary_set_with_own_descriptor(
  o: &JsObject,
  p: PropertyKey,
  v: Value,
  receiver: &Value,
  mut own_desc: Option<PropertyDescriptor>,
) -> Result<bool, Value> {
  // 1. If ownDesc is undefined, then
  if own_desc.is_none() {
    // a. Let parent be ? O.[[GetPrototypeOf]]().
    let parent = o.get_prototype_of()?;
    // b. If parent is not null, then
    if let Prototype::Object(parent) = parent {
      // i. Return ? parent.[[Set]](P, V, Receiver).
      return parent.set(p, v, receiver);
    // c. Else,
    } else {
      // i. Set ownDesc to the PropertyDescriptor { [[Value]]: undefined, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true }.
      own_desc = Some(
        PropertyDescriptor::empty()
          .value(Value::Undefined)
          .writable(true)
          .enumerable(true)
          .configurable(true),
      );
    }
  }
  let own_desc = own_desc.unwrap();
  // 2. If IsDataDescriptor(ownDesc) is true, then
  if own_desc.is_data_descriptor() {
    // a. If ownDesc.[[Writable]] is false, return false.
    if let Some(false) = own_desc.writable {
      return Ok(false);
    }
    // b. If Type(Receiver) is not Object, return false.
    match receiver {
      Value::Object(receiver) => {
        // c. Let existingDescriptor be ? Receiver.[[GetOwnProperty]](P).
        let existing_descriptor = receiver.get_own_property(&p)?;
        // d. If existingDescriptor is not undefined, then
        if let Some(existing_descriptor) = existing_descriptor {
          // i. If IsAccessorDescriptor(existingDescriptor) is true, return false.
          if existing_descriptor.is_accessor_descriptor() {
            return Ok(false);
          }
          // ii. If existingDescriptor.[[Writable]] is false, return false.
          if let Some(false) = existing_descriptor.writable {
            return Ok(false);
          }
          // iii. Let valueDesc be the PropertyDescriptor { [[Value]]: V }.
          let value_desc = PropertyDescriptor::empty().value(v);
          // iv. Return ? Receiver.[[DefineOwnProperty]](P, valueDesc).
          return receiver.define_own_property(p, value_desc);
        // e. Else,
        } else {
          // i. Assert: Receiver does not currently have a property P.
          // ii. Return ? CreateDataProperty(Receiver, P, V).
          return create_data_property(receiver, p, v);
        }
      }
      _ => return Ok(false),
    }
  }
  // 3. Assert: IsAccessorDescriptor(ownDesc) is true.
  assert!(own_desc.is_accessor_descriptor());
  // 4. Let setter be ownDesc.[[Set]].
  let setter = own_desc.set.unwrap();
  match setter {
    // 5. If setter is undefined, return false.
    GetSet::Undefined => return Ok(false),
    GetSet::Object(setter) => {
      // 6. Perform ? Call(setter, Receiver, « V »).
      call(&Value::Object(setter), receiver, &[v])?;
    }
  }
  // 7. Return true.
  Ok(true)
}

/// https://tc39.es/ecma262/#sec-ordinarydelete
pub fn ordinary_delete(o: &JsObject, p: &PropertyKey) -> Result<bool, Value> {
  // 1. Let desc be ? O.[[GetOwnProperty]](P).
  let desc = o.get_own_property(p)?;
  match desc {
    // 2. If desc is undefined, return true.
    None => return Ok(true),
    Some(desc) => {
      // 3. If desc.[[Configurable]] is true, then
      if let Some(true) = desc.configurable {
        // a. Remove the own property with name P from O.
        o.get_properties_mut().remove(p);
        // b. Return true.
        return Ok(true);
      }
    }
  }
  // 4. Return false.
  Ok(false)
}

/// https://tc39.es/ecma262/#sec-ordinaryownpropertykeys
pub fn ordinary_own_property_keys(
  o: &JsObject,
) -> Result<Vec<PropertyKey>, Value> {
  // 1. Let keys be a new empty List.
  let mut keys = Vec::new();
  // 2. For each own property key P of O such that P is an array index, in ascending numeric index order, do
  // a. Add P as the last element of keys.
  for p in o.get_properties().keys() {
    if is_array_index(&p.clone().into())? {
      keys.push(p.clone())
    }
  }
  todo!();
  // 3. For each own property key P of O such that Type(P) is String and P is not an array index, in ascending chronological order of property creation, do
  // a. Add P as the last element of keys.
  // 4. For each own property key P of O such that Type(P) is Symbol, in ascending chronological order of property creation, do
  // a. Add P as the last element of keys.
  // 5. Return keys.
  Ok(keys)
}
