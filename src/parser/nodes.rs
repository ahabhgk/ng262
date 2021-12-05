use super::source::Source;

pub struct Location {
  pub index: usize,
  pub line: usize,
  pub column: usize,
}

pub enum NodeType {
  IdentifierName { name: String },
}

pub struct Node<'i> {
  node_type: NodeType,
  start: Location,
  end: Location,
  is_strict: bool,
  source: &'i Source<'i>,
}

pub struct NodeBuilder {
  start: Location,
  is_strict: bool,
}

impl NodeBuilder {
  pub fn new(start: Location, is_strict: bool) -> Self {
    Self { start, is_strict }
  }

  pub fn build<'i>(
    self,
    end: Location,
    node_type: NodeType,
    source: &'i Source,
  ) -> Node<'i> {
    Node {
      node_type,
      start: self.start,
      end,
      is_strict: self.is_strict,
      source,
    }
  }
}

impl Node<'_> {
  pub fn start(location: Location, is_strict: bool) -> NodeBuilder {
    NodeBuilder::new(location, is_strict)
  }

  pub fn source_text() -> String {
    todo!()
  }
}
