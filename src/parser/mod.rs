use std::collections::HashSet;

use self::{error::SyntaxError, scope::Scope};

pub mod error;
pub mod lexer;
pub mod scope;
pub mod strict;
pub mod tokens;

struct State {
  has_top_level_await: bool,
  json: bool,
}

pub struct Parser {
  source: String,
  specifier: Option<String>,
  early_errors: HashSet<SyntaxError>,
  state: State,
  scope: Scope,
}

impl Parser {}
