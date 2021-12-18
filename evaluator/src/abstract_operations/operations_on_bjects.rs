use crate::{
  language_types::{undefined::JsUndefined, Value},
  specification_types::completion_record::Completion,
};

/// https://tc39.es/ecma262/#sec-call
pub fn call(
  f: &Value,
  v: &Value,
  arguments_list: &[Value],
) -> Result<Completion, Completion> {
  // 1. If argumentsList is not present, set argumentsList to a new empty List.
  // 2. If IsCallable(F) is false, throw a TypeError exception.
  if !f.is_callable() {
    return Err(Completion::throw(Value::Undefined(JsUndefined))); // TODO
  }
  // 3. Return ? F.[[Call]](V, argumentsList).
  // f.call(v, arguments_list)?
  todo!()
}
