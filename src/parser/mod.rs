use std::collections::HashSet;

use self::{
  error::{SyntaxError, SyntaxErrorInfo},
  lexer::Lexer,
  nodes::{Location, Node, NodeBuilder, NodeType},
  scope::Scope,
  strict::{Strict, UseStrict},
};

mod error;
mod identifier;
mod lexer;
mod nodes;
mod scope;
mod source;
mod strict;
mod tokens;

struct State {
  has_top_level_await: bool,
  json: bool,
}

pub struct Parser<'strict> {
  lexer: Lexer<'strict>,
  specifier: Option<String>,
  early_errors: HashSet<SyntaxError>,
  state: State,
  scope: Scope,
  // TODO: use derive marco
  strict: &'strict mut Strict,
}

impl UseStrict for Parser<'_> {
  fn is_strict(&self) -> bool {
    self.strict.is_strict()
  }

  fn use_strict(&mut self, is_strict: bool) {
    self.strict.use_strict(is_strict);
  }
}

impl SyntaxErrorInfo for Parser<'_> {
  fn line(&self) -> usize {
    self.lexer.line()
  }

  fn index(&self) -> usize {
    self.lexer.index()
  }

  fn get(&self, index: usize) -> Option<char> {
    self.lexer.get(index)
  }

  fn slice(&self, start_index: usize, end_index: usize) -> String {
    self.lexer.slice(start_index, end_index)
  }
}

impl Parser<'_> {
  fn start(&mut self) -> Result<NodeBuilder, SyntaxError> {
    let peek = self.lexer.peek()?;
    let location = Location {
      index: peek.start_index,
      line: peek.line,
      column: peek.column,
    };
    Ok(NodeBuilder::new(location, self.is_strict()))
  }

  fn finish(&mut self, node: NodeBuilder, node_type: NodeType) -> Node {
    let current = self.lexer.current();
    let index = current.end_index;
    let location = Location {
      index,
      line: current.line,
      column: current.column,
    };
    let source_text = self.lexer.get_source().slice(node.start.index, index);
    node.build(location, node_type, source_text)
  }
}
