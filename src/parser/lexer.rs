use std::{
  iter::Peekable,
  str::{CharIndices, Chars},
};

use super::{
  error::{SyntaxError, SyntaxErrorTemplate},
  tokens::{Token, TokenType, TokenValue},
};

fn is_line_terminator(c: char) -> bool {
  match c {
    '\r' | '\n' | '\u{2028}' | '\u{2029}' => true,
    _ => false,
  }
}

fn is_whitespace(c: char) -> bool {
  match c {
    '\u{0009}' | '\u{000b}' | '\u{000c}' | '\u{0020}' | '\u{00a0}'
    | '\u{feff}' => true,
    _ if c.is_whitespace() => true,
    _ => false,
  }
}

fn is_decimal_digit(c: char) -> bool {
  c.is_digit(10)
}

struct Input<'a> {
  iter: Chars<'a>,
  index: usize,
}

impl<'a> Input<'a> {
  pub fn new(source: &'a str) -> Self {
    Self {
      iter: source.chars(),
      index: 0,
    }
  }

  pub fn position(&self) -> usize {
    self.index
  }

  pub fn current(&self) -> Option<char> {
    self.get(self.index)
  }

  pub fn peek(&self) -> Option<char> {
    self.get(self.index + 1)
  }

  pub fn skip(&mut self, num: usize) {
    self.index += num;
  }

  pub fn next(&mut self) -> Option<char> {
    self.skip(1);
    self.current()
  }

  pub fn get(&self, i: usize) -> Option<char> {
    self.iter.clone().nth(i)
  }

  pub fn slice(&self, start: usize, end: usize) -> String {
    let str = self.iter.as_str();
    let str = &str[start..end];
    str.to_owned()
  }
}

pub struct Lexer<'a> {
  source: Input<'a>,
  line: usize,
  column_offset: usize,
  // scanned_value: Option<>
  line_terminator_before_next_token: bool,
  escape_index: isize,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Self {
      source: Input::new(source),
      line: 1,
      column_offset: 0,
      line_terminator_before_next_token: false,
      escape_index: -1,
    }
  }

  fn create_token(
    &self,
    r#type: TokenType,
    start_index: usize,
    line: usize,
    column: usize,
  ) -> Token {
    Token {
      r#type,
      value: TokenValue {},
      start_index,
      end_index: self.source.position(),
      line,
      column,
      had_line_terminator_before: self.line_terminator_before_next_token,
      escaped: self.escape_index != -1,
    }
  }

  fn next_token(&mut self) -> Result<Token, SyntaxError> {
    self.skip_space()?;

    // set token location info after skipping space
    let position = self.source.position();
    let position_for_next_token = position;
    let line_for_next_token = self.line;
    let column_for_next_token = position - self.column_offset + 1;

    let c = self.source.current();
    if c.is_none() {
      return Ok(self.create_token(
        TokenType::EOS,
        position_for_next_token,
        line_for_next_token,
        column_for_next_token,
      ));
    }
    let c = c.unwrap();
    match c {
      '(' | ')' | '{' | '}' | '[' | ']' | ':' | ';' | ',' | '~' | '`' => {
        self.source.skip(1);
        return Ok(self.create_token(
          TokenType::from_single(c),
          position_for_next_token,
          line_for_next_token,
          column_for_next_token,
        ));
      }
      // ? ?. ?? ??=
      '?' => match self.source.next() {
        Some('.') => {
          if let Some(c1) = self.source.peek() {
            if !is_decimal_digit(c1) {
              self.source.skip(1);
              return Ok(self.create_token(
                TokenType::OPTIONAL,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
          }
        }
        Some('?') => {
          if let Some(c1) = self.source.next() {
            if c1 == '=' {
              self.source.skip(1);
              return Ok(self.create_token(
                TokenType::ASSIGN_NULLISH,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
          }
          return Ok(self.create_token(
            TokenType::NULLISH,
            position_for_next_token,
            line_for_next_token,
            column_for_next_token,
          ));
        }
        _ => {
          return Ok(self.create_token(
            TokenType::CONDITIONAL,
            position_for_next_token,
            line_for_next_token,
            column_for_next_token,
          ))
        }
      },
      // < <= << <<=
      '<' => match self.source.next() {
        
      },
      _ => {
        return Err(
          self.create_syntax_error(
            position,
            SyntaxErrorTemplate::UnexpectedToken,
          ),
        )
      }
    }

    todo!()
  }

  /// Skip comments or whitespaces.
  ///
  /// https://tc39.es/ecma262/#sec-white-space
  fn skip_space(&mut self) -> Result<(), SyntaxError> {
    while let Some(c) = self.source.current() {
      match c {
        ' ' | '\t' => {
          self.source.skip(1);
        }
        '/' => match self.source.peek() {
          Some('/') => self.skip_line_comment(),
          Some('*') => self.skip_block_comment()?,
          _ => return Ok(()),
        },
        _ => {
          if is_whitespace(c) {
            self.source.skip(1);
          } else if is_line_terminator(c) {
            self.terminate_line(c);
          } else {
            return Ok(());
          }
        }
      }
    }
    Ok(())
  }

  fn terminate_line(&mut self, c: char) {
    self.source.skip(1);
    if c == '\r' && self.source.current() == Some('\n') {
      self.source.skip(1);
    }
    self.line += 1;
    self.column_offset = self.source.position();
    self.line_terminator_before_next_token = true;
  }

  fn skip_line_comment(&mut self) {
    self.source.skip(2);
    while let Some(c) = self.source.current() {
      if is_line_terminator(c) {
        self.terminate_line(c)
      } else {
        self.source.skip(1);
      }
    }
  }

  fn skip_block_comment(&mut self) -> Result<(), SyntaxError> {
    let position = self.source.position();
    self.source.skip(2);
    while let Some(c) = self.source.current() {
      match self.source.peek() {
        Some(p) => {
          if c == '*' && p == '/' {
            self.source.skip(2);
            return Ok(());
          }
          if is_line_terminator(c) {
            self.terminate_line(c);
          } else {
            self.source.skip(1);
          }
        }
        None => {
          return Err(self.create_syntax_error(
            position,
            SyntaxErrorTemplate::UnterminatedComment,
          ))
        }
      }
    }
    Ok(())
  }

  fn create_syntax_error(
    &self,
    position: usize,
    template: SyntaxErrorTemplate,
  ) -> SyntaxError {
    let start_index = position;
    let end_index = position + 1;
    let line = self.line;

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
    while let Some(c) = self.source.get(line_start) {
      if !is_line_terminator(c) {
        line_start -= 1;
      }
    }

    let mut line_end = start_index;
    while let Some(c) = self.source.get(line_end) {
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
      self.source.slice(line_start, line_end),
      " ".repeat(start_index - line_start),
      "^".repeat(1.max(end_index - start_index)),
    );
    SyntaxError {
      message,
      decoration,
    }
  }
}
