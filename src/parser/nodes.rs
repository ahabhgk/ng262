use super::source::SourceText;

pub struct Location {
  pub index: usize,
  pub line: usize,
  pub column: usize,
}

pub enum NodeType {
  IdentifierName { name: String },
}

pub struct Node {
  node_type: NodeType,
  start: Location,
  end: Location,
  is_strict: bool,
  source_text: String,
}

pub struct NodeBuilder {
  pub start: Location,
  pub is_strict: bool,
}

impl NodeBuilder {
  pub fn new(start: Location, is_strict: bool) -> Self {
    Self { start, is_strict }
  }

  pub fn build(
    self,
    end: Location,
    node_type: NodeType,
    source_text: String,
  ) -> Node {
    Node {
      node_type,
      start: self.start,
      end,
      is_strict: self.is_strict,
      source_text,
    }
  }
}

impl SourceText for Node {
  fn source_text(&self) -> &str {
    self.source_text.as_str()
  }
}

impl Node {
  pub fn start(location: Location, is_strict: bool) -> NodeBuilder {
    NodeBuilder::new(location, is_strict)
  }
}
