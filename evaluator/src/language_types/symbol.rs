/// https://tc39.es/ecma262/#sec-ecmascript-language-types-symbol-type
#[derive(Debug, PartialEq, Eq)]
pub struct JsSymbol {
  id: usize,
}
