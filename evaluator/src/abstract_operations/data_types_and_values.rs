use crate::language_types::Value;

/// https://tc39.es/ecma262/#integral-number
pub fn is_integer_index(v: &Value) -> Result<bool, Value> {
  match v {
    v @ Value::String(_) => {
      let numeric = v.canonical_numeric_index_string()?;
      if numeric.is_none() {
        return Ok(false);
      }
      let numeric = numeric.unwrap();
      if numeric == 0.0 && numeric.is_sign_positive() {
        return Ok(true);
      }
      Ok(numeric.is_sign_positive() && numeric < 9007199254740991.0)
    }
    _ => Ok(false),
  }
}

/// https://tc39.es/ecma262/#array-index
pub fn is_array_index(v: &Value) -> Result<bool, Value> {
  match v {
    v @ Value::String(_) => {
      let numeric = v.canonical_numeric_index_string()?;
      if numeric.is_none() {
        return Ok(false);
      }
      let numeric = numeric.unwrap();
      if numeric == 0.0 && numeric.is_sign_positive() {
        return Ok(true);
      }
      Ok(numeric.is_sign_positive() && numeric < 2_f64.powi(32) - 1.0)
    }
    _ => Ok(false),
  }
}
