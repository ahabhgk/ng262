use std::{error::Error, fmt};

use super::lexer::is_line_terminator;

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
  pub fn from_index<S: SyntaxErrorInfo>(
    informer: &S,
    offset: isize,
    template: SyntaxErrorTemplate,
  ) -> SyntaxError {
    let index = informer.index();
    let start_index = index as isize + offset;
    if start_index < 0 {
      panic!("index is out of range, (index + offset) should not < 0");
    }
    let start_index = start_index as usize;
    let end_index = index + 1;
    let line = informer.line();

    /*
     * Source looks like:
     *
     *  const a = 1;
     *  const b 'string string string'; // a string
     *  const c = 3;                  |            |
     *  |       |                     |            |
     *  |       | startIndex          | endIndex   |
     *  | lineStart                                | lineEnd
     *
     * Exception looks like:
     *
     *  const b 'string string string'; // a string
     *          ^^^^^^^^^^^^^^^^^^^^^^
     *  SyntaxError: unexpected token
     */

    let mut line_start = start_index;
    while let Some(c) = informer.get(line_start) {
      if !is_line_terminator(c) {
        line_start -= 1;
      }
    }

    let mut line_end = start_index;
    while let Some(c) = informer.get(line_end) {
      if !is_line_terminator(c) {
        line_end += 1;
      }
    }

    let column = start_index - line_start + 1;
    let message = format!("{}", template);
    // TODO: specifier
    let decoration = format!(
      "\n{}:{}\n{}\n{}{}",
      line,
      column,
      informer.slice(line_start, line_end),
      " ".repeat(start_index - line_start),
      "^".repeat(1.max(end_index - start_index)),
    );
    SyntaxError {
      message,
      decoration,
    }
  }
}

pub trait SyntaxErrorInfo {
  fn index(&self) -> usize;

  fn line(&self) -> usize;

  fn get(&self, index: usize) -> Option<char>;

  fn slice(&self, start_index: usize, end_index: usize) -> String {
    let mut s = String::new();
    for i in start_index..end_index {
      s.push(self.get(i).expect("cursor out of range"));
    }
    s
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
