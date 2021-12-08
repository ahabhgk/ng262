use crate::parser::{strict::IsStrict, tokens::is_reserved_word_strict};

use super::{
  error::{EarlyError, ParseError, SyntaxError, SyntaxErrorTemplate},
  nodes::{Node, NodeType},
  resolver::Flag,
  tokens::TokenType,
  Parser,
};

impl Parser {
  // IdentifierName
  fn parse_identifier_name(&mut self) -> Result<Node, ParseError> {
    let node = self.start()?;
    let peek = self.lexer.peek()?;
    if matches!(
      peek.token_type,
      TokenType::Identifier(_) | TokenType::EscapedKeyword(_)
    ) || peek.token_type.is_keyword()
    {
      let name = self.lexer.next()?.token_type.identifier_or_keyword_value();
      Ok(self.finish(node, NodeType::IdentifierName { name }))
    } else {
      let peek = peek.to_owned();
      Err(
        SyntaxError::from_token(
          self,
          &peek,
          SyntaxErrorTemplate::UnexpectedToken,
        )
        .into(),
      )
    }
  }

  // BindingIdentifier :
  //   Identifier
  //   `yield`
  //   `await`
  fn parse_binding_identifier(&mut self) -> Result<Node, ParseError> {
    let node = self.start()?;
    let token = self.lexer.next()?.to_owned();
    let name = match &token.token_type {
      TokenType::Identifier(name) => name.clone(),
      TokenType::EscapedKeyword(name) => name.clone(),
      TokenType::Yield => "yield".to_owned(),
      TokenType::Await => "await".to_owned(), // TODO: arrowInfoStack
      _ => {
        return Err(
          SyntaxError::from_token(
            self,
            &token,
            SyntaxErrorTemplate::UnexpectedToken,
          )
          .into(),
        )
      }
    };
    if (name == "yield" || name == "await")
      && (self.resolver.flags.has(Flag::Yield)
        || self.resolver.flags.has(Flag::Module))
    {
      return Err(
        EarlyError::from(SyntaxError::from_token(
          self,
          &token,
          SyntaxErrorTemplate::UnexpectedReservedWordStrict,
        ))
        .into(),
      );
    }
    if self.resolver.is_strict() {
      if is_reserved_word_strict(&name) {
        return Err(
          EarlyError::from(SyntaxError::from_token(
            self,
            &token,
            SyntaxErrorTemplate::UnexpectedReservedWordStrict,
          ))
          .into(),
        );
      }
      if name == "eval" || name == "argument" {
        return Err(
          EarlyError::from(SyntaxError::from_token(
            self,
            &token,
            SyntaxErrorTemplate::UnexpectedEvalOrArguments,
          ))
          .into(),
        );
      }
    }
    if name != "yield" && name != "await" && token.token_type.is_keyword() {
      return Err(
        EarlyError::from(SyntaxError::from_token(
          self,
          &token,
          SyntaxErrorTemplate::UnexpectedToken,
        ))
        .into(),
      );
    }
    Ok(self.finish(node, NodeType::BindingIdentifier { name }))
  }
}
