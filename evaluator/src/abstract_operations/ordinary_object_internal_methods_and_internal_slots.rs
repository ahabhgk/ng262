use crate::language_types::object::{JsObject, Prototype};

/// https://tc39.es/ecma262/#sec-ordinarygetprototypeof
pub fn ordinary_get_prototype_of(o: JsObject) -> Prototype {
  // 1. Return O.[[Prototype]].
  o.get_prototype()
}
