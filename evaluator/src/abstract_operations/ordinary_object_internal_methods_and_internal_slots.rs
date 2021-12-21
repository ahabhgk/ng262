use crate::{
  language_types::object::{JsObject, PropertyKey, Prototype},
  specification_types::property_descriptor::PropertyDescriptor,
};

/// https://tc39.es/ecma262/#sec-ordinarygetprototypeof
pub fn ordinary_get_prototype_of(o: &JsObject) -> Prototype {
  // 1. Return O.[[Prototype]].
  o.prototype().clone()
}

/// https://tc39.es/ecma262/#sec-ordinarysetprototypeof
pub fn ordinary_set_prototype_of(o: &JsObject, v: Prototype) -> bool {
  // 1. Let current be O.[[Prototype]].
  let current = o.prototype().clone();
  // 2. If SameValue(V, current) is true, return true.
  if v == current {
    return true;
  }
  // 3. Let extensible be O.[[Extensible]].
  let extensible = o.extensible();
  // 4. If extensible is false, return false.
  if extensible.into() {
    return false;
  }
  // 5. Let p be V.
  let mut p = v.clone();
  // 6. Let done be false.
  // 7. Repeat, while done is false,
  // a. If p is null, set done to true.
  while let Prototype::Object(proto) = p {
    // b. Else if SameValue(p, O) is true, return false.
    if proto == *o {
      return false;
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
  true
}

/// https://tc39.es/ecma262/#sec-ordinaryisextensible
pub fn ordinary_is_extensible(o: &JsObject) -> bool {
  // 1. Return O.[[Extensible]].
  o.extensible()
}

/// https://tc39.es/ecma262/#sec-ordinarypreventextensions
pub fn ordinary_prevent_extensions(o: &JsObject) -> bool {
  // 1. Set O.[[Extensible]] to false.
  o.set_extensible(false);
  // 2. Return true.
  true
}

/// https://tc39.es/ecma262/#sec-ordinarygetownproperty
pub fn ordinary_get_own_property(
  o: &JsObject,
  p: &PropertyKey,
) -> Option<PropertyDescriptor> {
  // 1. If O does not have an own property with key P, return undefined.
  match o.get_properties().get(p) {
    None => None,
    Some(x) => {
      // 2. Let D be a newly created Property Descriptor with no fields.
      // 3. Let X be O's own property whose key is P.
      let mut d = PropertyDescriptor::empty();
      // 4. If X is a data property, then
      if x.is_data_descriptor() {
        // a. Set D.[[Value]] to the value of X's [[Value]] attribute.
        d.value = x.value.clone();
        // b. Set D.[[Writable]] to the value of X's [[Writable]] attribute.
        d.writable = x.writable.clone();
        // 5. Else,
        // a. Assert: X is an accessor property.
      } else if x.is_accessor_descriptor() {
        // b. Set D.[[Get]] to the value of X's [[Get]] attribute.
        d.get = x.get.clone();
        // c. Set D.[[Set]] to the value of X's [[Set]] attribute.
        d.set = x.set.clone();
      }
      // 6. Set D.[[Enumerable]] to the value of X's [[Enumerable]] attribute.
      d.enumerable = x.enumerable.clone();
      // 7. Set D.[[Configurable]] to the value of X's [[Configurable]] attribute.
      d.configurable = x.configurable.clone();
      // 8. Return D.
      Some(d)
    }
  }
}
