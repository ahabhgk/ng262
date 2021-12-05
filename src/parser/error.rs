use std::{error::Error, fmt};

use super::{lexer::is_line_terminator, tokens::Token};

/// SyntaxError
///
/// Source looks like:
/// ```js
///  const a = 1;
///  const b 'string string string'; // a string
///  const c = 3;                  |            |
///  |       |                     |            |
///  |       | startIndex          | endIndex   |
///  | lineStart                                | lineEnd
/// ```
///
/// Exception looks like:
///
/// ```js
///  const b 'string string string'; // a string
///          ^^^^^^^^^^^^^^^^^^^^^^
///  SyntaxError: unexpected token
/// ```
#[derive(Debug, Clone)]
pub struct SyntaxError {
  message: String,
  decoration: String,
}

impl Error for SyntaxError {}

impl fmt::Display for SyntaxError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "SyntaxError")
  }
}

impl SyntaxError {
  #[allow(clippy::too_many_arguments)]
  fn new<S: SyntaxErrorInfo>(
    informer: &S,
    template: SyntaxErrorTemplate,
    start_index: usize,
    end_index: usize,
    line_start: usize,
    line_end: usize,
    line: usize,
    column: usize,
  ) -> Self {
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

  fn line_start_index<S: SyntaxErrorInfo>(
    informer: &S,
    start_index: usize,
  ) -> usize {
    let mut line_start = start_index;
    while let Some(c) = informer.get(line_start) {
      if !is_line_terminator(c) {
        line_start -= 1;
      }
    }
    line_start
  }

  fn line_end_index<S: SyntaxErrorInfo>(
    informer: &S,
    start_index: usize,
  ) -> usize {
    let mut line_end = start_index;
    while let Some(c) = informer.get(line_end) {
      if !is_line_terminator(c) {
        line_end += 1;
      }
    }
    line_end
  }

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
    let line_start = Self::line_start_index(informer, start_index);
    let line_end = Self::line_end_index(informer, start_index);
    let line = informer.line();
    let column = start_index - line_start + 1;

    Self::new(
      informer,
      template,
      start_index,
      end_index,
      line_start,
      line_end,
      line,
      column,
    )
  }

  pub fn from_token<S: SyntaxErrorInfo>(
    informer: &S,
    token: &Token,
    template: SyntaxErrorTemplate,
  ) -> Self {
    let start_index = token.start_index;
    let end_index = token.end_index;
    let line_start = Self::line_start_index(informer, start_index);
    let line_end = Self::line_end_index(informer, start_index);
    let line = token.line;
    let column = token.column;

    Self::new(
      informer,
      template,
      start_index,
      end_index,
      line_start,
      line_end,
      line,
      column,
    )
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
