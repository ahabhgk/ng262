use std::collections::HashSet;

use self::{error::SyntaxError, scope::Scope};

pub mod error;
pub mod lexer;
pub mod scope;
pub mod tokens;

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

impl Parser {
  fn is_strict_mode(&self) -> bool {
    self.state.strict
  }
}
