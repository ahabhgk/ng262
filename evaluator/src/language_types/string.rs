pub type JsString = String;

/// https://tc39.es/ecma262/#sec-stringtonumber
pub fn string_to_number(str: &str) -> f64 {
  // 1. Let text be ! StringToCodePoints(str).
  // 2. Let literal be ParseText(text, StringNumericLiteral).
  // 3. If literal is a List of errors, return NaN.
  // 4. Return StringNumericValue of literal.
  str.parse().unwrap_or(f64::NAN)
}
