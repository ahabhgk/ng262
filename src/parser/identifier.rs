use super::{
  error::{SyntaxError, SyntaxErrorTemplate},
  nodes::{Node, NodeType},
  tokens::TokenType,
  Parser,
};

impl Parser<'_, '_> {
  // IdentifierName
  fn parse_identifier_name(&mut self) -> Result<Node, SyntaxError> {
    let node = self.start()?;
    let peek = self.lexer.peek()?;
    if matches!(
      peek.token_type,
      TokenType::IDENTIFIER(_) | TokenType::ESCAPED_KEYWORD(_)
    ) || peek.token_type.is_keyword()
    {
      let name = self.lexer.next()?.token_type.identifier_or_keyword_value();
      let node_type = NodeType::IdentifierName { name };
      let node = self.finish(node, node_type);
      Ok(node)
    } else {
      let peek = &self.lexer.peek()?;
      Err(SyntaxError::from_token(
        self,
        peek,
        SyntaxErrorTemplate::UnexpectedToken,
      ))
    }
  }
}
