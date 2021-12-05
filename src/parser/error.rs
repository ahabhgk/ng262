use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct SyntaxError {
  pub message: String,
  pub decoration: String,
}

impl Error for SyntaxError {}

impl fmt::Display for SyntaxError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "SyntaxError")
  }
}

impl SyntaxError {
  pub fn from_position() -> Self {
    todo!()
  }
}

#[derive(Debug)]
pub enum SyntaxErrorTemplate {
  UnterminatedComment,
  UnexpectedToken,
  InvalidUnicodeEscape,
  InvalidCodePoint,
  UnterminatedString,
  IllegalOctalEscape,
}

impl fmt::Display for SyntaxErrorTemplate {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnterminatedComment => write!(f, "Missing */ after comment"),
      Self::UnexpectedToken => write!(f, "Unexpected token"),
      Self::InvalidUnicodeEscape => write!(f, "Invalid unicode escape"),
      Self::InvalidCodePoint => write!(f, "Not a valid code point"),
      Self::UnterminatedString => {
        write!(f, "Missing \' or \" after string literal")
      }
      Self::IllegalOctalEscape => write!(f, "Illegal octal escape"),
    }
  }
}
