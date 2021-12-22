use crate::{
  language_types::{object::JsObject, Value},
  specification_types::completion_record::Completion,
};

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
