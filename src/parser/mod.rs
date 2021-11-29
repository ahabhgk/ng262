use std::collections::HashSet;

use self::scope::Scope;

pub mod lexer;
pub mod scope;
pub mod tokens;

pub struct SyntaxError {}

struct State {
  has_top_level_await: bool,
  strict: bool,
  json: bool,
}

pub struct Parser {
  source: String,
  specifier: Option<String>,
  earlyErrors: HashSet<SyntaxError>,
  state: State,
  scope: Scope,
}
