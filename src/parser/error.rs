use std::{error::Error, fmt};

#[derive(Debug)]
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

#[derive(Debug)]
pub enum SyntaxErrorTemplate {
  UnterminatedComment,
  UnexpectedToken,
  InvalidUnicodeEscape,
  InvalidCodePoint,
}

impl fmt::Display for SyntaxErrorTemplate {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnterminatedComment => write!(f, "Missing */ after comment"),
      Self::UnexpectedToken => write!(f, "Unexpected token"),
      Self::InvalidUnicodeEscape => write!(f, "Invalid unicode escape"),
      Self::InvalidCodePoint => write!(f, "Not a valid code point"),
    }
  }
}
