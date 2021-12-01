use std::str::Chars;

use unicode_xid::UnicodeXID;

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

fn is_identifier_start(c: char) -> bool {
  match c {
    'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l'
    | 'm' | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' | 'w' | 'x'
    | 'y' | 'z' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J'
    | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V'
    | 'W' | 'X' | 'Y' | 'Z' | '$' | '_' | '\\' => true,
    _ if c.is_xid_start() => true,
    _ => false,
  }
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

  pub fn forward(&mut self) {
    self.index += 1;
  }

  pub fn backward(&mut self) {
    self.index -= 1;
  }

  pub fn next(&mut self) -> Option<char> {
    self.forward();
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

    // fast path for usual case
    if c < char::from(127) {
      match c {
        '(' | ')' | '{' | '}' | '[' | ']' | ':' | ';' | ',' | '~' | '`' => {
          self.source.forward();
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
            if let Some(c) = self.source.peek() {
              if !is_decimal_digit(c) {
                self.source.forward();
                return Ok(self.create_token(
                  TokenType::OPTIONAL,
                  position_for_next_token,
                  line_for_next_token,
                  column_for_next_token,
                ));
              }
            }
          }
          Some('?') => match self.source.next() {
            Some('=') => {
              self.source.forward();
              return Ok(self.create_token(
                TokenType::ASSIGN_NULLISH,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
            _ => {
              return Ok(self.create_token(
                TokenType::NULLISH,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ))
            }
          },
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
          Some('=') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::LTE,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          Some('<') => match self.source.next() {
            Some('=') => {
              self.source.forward();
              return Ok(self.create_token(
                TokenType::ASSIGN_SHL,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
            _ => {
              return Ok(self.create_token(
                TokenType::SHL,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ))
            }
          },
          _ => {
            return Ok(self.create_token(
              TokenType::LT,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // > >= >> >>= >>> >>>=
        '>' => match self.source.next() {
          Some('=') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::GTE,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          Some('>') => match self.source.next() {
            Some('>') => match self.source.next() {
              Some('=') => {
                self.source.forward();
                return Ok(self.create_token(
                  TokenType::ASSIGN_SHR,
                  position_for_next_token,
                  line_for_next_token,
                  column_for_next_token,
                ));
              }
              _ => {
                return Ok(self.create_token(
                  TokenType::SHR,
                  position_for_next_token,
                  line_for_next_token,
                  column_for_next_token,
                ));
              }
            },
            Some('=') => {
              self.source.forward();
              return Ok(self.create_token(
                TokenType::ASSIGN_SAR,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
            _ => {
              return Ok(self.create_token(
                TokenType::SAR,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
          },
          _ => {
            return Ok(self.create_token(
              TokenType::GT,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
        },
        // = == === =>
        '=' => match self.source.next() {
          Some('=') => match self.source.next() {
            Some('=') => {
              self.source.forward();
              return Ok(self.create_token(
                TokenType::EQ_STRICT,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
            _ => {
              return Ok(self.create_token(
                TokenType::EQ,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ))
            }
          },
          Some('>') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::ARROW,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          _ => {
            return Ok(self.create_token(
              TokenType::ASSIGN,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // ! != !==
        '!' => match self.source.next() {
          Some('=') => match self.source.next() {
            Some('=') => {
              self.source.forward();
              return Ok(self.create_token(
                TokenType::NE_STRICT,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
            _ => {
              return Ok(self.create_token(
                TokenType::NE,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ))
            }
          },
          _ => {
            return Ok(self.create_token(
              TokenType::NOT,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // + ++ +=
        '+' => match self.source.next() {
          Some('+') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::INC,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          Some('=') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::ASSIGN_ADD,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          _ => {
            return Ok(self.create_token(
              TokenType::ADD,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // - -- -=
        '-' => match self.source.next() {
          Some('-') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::DEC,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          Some('=') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::ASSIGN_SUB,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          _ => {
            return Ok(self.create_token(
              TokenType::SUB,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // * *= ** **=
        '*' => match self.source.next() {
          Some('=') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::ASSIGN_MUL,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          Some('*') => match self.source.next() {
            Some('=') => {
              self.source.forward();
              return Ok(self.create_token(
                TokenType::ASSIGN_EXP,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
            _ => {
              return Ok(self.create_token(
                TokenType::EXP,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ))
            }
          },
          _ => {
            return Ok(self.create_token(
              TokenType::MUL,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // % %=
        '%' => match self.source.next() {
          Some('=') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::ASSIGN_MOD,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          _ => {
            return Ok(self.create_token(
              TokenType::MOD,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // / /=
        '/' => match self.source.next() {
          Some('=') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::ASSIGN_DIV,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          _ => {
            return Ok(self.create_token(
              TokenType::DIV,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // & && &= &&=
        '&' => match self.source.next() {
          Some('&') => match self.source.next() {
            Some('=') => {
              self.source.forward();
              return Ok(self.create_token(
                TokenType::ASSIGN_AND,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
            _ => {
              return Ok(self.create_token(
                TokenType::AND,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ))
            }
          },
          Some('=') => {
            return Ok(self.create_token(
              TokenType::ASSIGN_BIT_AND,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
          _ => {
            return Ok(self.create_token(
              TokenType::BIT_AND,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // | || |= ||=
        '|' => match self.source.next() {
          Some('|') => match self.source.next() {
            Some('=') => {
              self.source.forward();
              return Ok(self.create_token(
                TokenType::ASSIGN_OR,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
            _ => {
              return Ok(self.create_token(
                TokenType::OR,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ))
            }
          },
          Some('=') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::ASSIGN_BIT_OR,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          _ => {
            return Ok(self.create_token(
              TokenType::BIT_OR,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // ^ ^=
        '^' => match self.source.next() {
          Some('=') => {
            self.source.forward();
            return Ok(self.create_token(
              TokenType::ASSIGN_BIT_XOR,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ));
          }
          _ => {
            return Ok(self.create_token(
              TokenType::BIT_XOR,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        // . ... NUMBER
        '.' => match self.source.next() {
          Some('.') => {
            if let Some('.') = self.source.next() {
              self.source.forward();
              return Ok(self.create_token(
                TokenType::ELLIPSIS,
                position_for_next_token,
                line_for_next_token,
                column_for_next_token,
              ));
            }
          }
          Some(c) if is_decimal_digit(c) => {
            self.source.backward();
            return self.scan_number();
          }
          _ => {
            return Ok(self.create_token(
              TokenType::PERIOD,
              position_for_next_token,
              line_for_next_token,
              column_for_next_token,
            ))
          }
        },
        '"' | '\'' => return self.scan_string(c),
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
          self.source.backward();
          return self.scan_number();
        }
        'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k'
        | 'l' | 'm' | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v'
        | 'w' | 'x' | 'y' | 'z' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G'
        | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q' | 'R'
        | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | '$' | '_' | '\\' => {
          return self.scan_identifier_or_keyword(false)
        }
        '#' => return self.scan_identifier_or_keyword(true),
        _ => {
          return Err(self.create_syntax_error(
            position,
            SyntaxErrorTemplate::UnexpectedToken,
          ))
        }
      }
    }

    if is_identifier_start(c) {
      return self.scan_identifier_or_keyword(false);
    }

    return Err(
      self.create_syntax_error(position, SyntaxErrorTemplate::UnexpectedToken),
    );
  }

  /// See https://tc39.es/ecma262/#sec-literals-numeric-literals
  fn scan_number(&self) -> Result<Token, SyntaxError> {
    todo!()
  }

  /// See https://tc39.es/ecma262/#sec-literals-string-literals
  fn scan_string(&self, quote: char) -> Result<Token, SyntaxError> {
    todo!()
  }

  /// See https://tc39.es/ecma262/#sec-names-and-keywords
  fn scan_identifier_or_keyword(
    &self,
    is_private: bool,
  ) -> Result<Token, SyntaxError> {
    todo!()
  }

  /// Skip comments or whitespaces.
  ///
  /// See https://tc39.es/ecma262/#sec-white-space, https://tc39.es/ecma262/#sec-comments
  fn skip_space(&mut self) -> Result<(), SyntaxError> {
    while let Some(c) = self.source.current() {
      match c {
        ' ' | '\t' => {
          self.source.forward();
        }
        '/' => match self.source.peek() {
          Some('/') => self.skip_line_comment(),
          Some('*') => self.skip_block_comment()?,
          _ => return Ok(()),
        },
        _ => {
          if is_whitespace(c) {
            self.source.forward();
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

  /// See https://tc39.es/ecma262/#sec-line-terminators
  fn terminate_line(&mut self, c: char) {
    self.source.forward();
    if c == '\r' && self.source.current() == Some('\n') {
      self.source.forward();
    }
    self.line += 1;
    self.column_offset = self.source.position();
    self.line_terminator_before_next_token = true;
  }

  fn skip_line_comment(&mut self) {
    self.source.forward();
    self.source.forward();
    while let Some(c) = self.source.current() {
      if is_line_terminator(c) {
        self.terminate_line(c)
      } else {
        self.source.forward();
      }
    }
  }

  fn skip_block_comment(&mut self) -> Result<(), SyntaxError> {
    let position = self.source.position();
    self.source.forward();
    self.source.forward();
    while let Some(c) = self.source.current() {
      match self.source.peek() {
        Some(p) => {
          if c == '*' && p == '/' {
            self.source.forward();
            self.source.forward();
            return Ok(());
          }
          if is_line_terminator(c) {
            self.terminate_line(c);
          } else {
            self.source.forward();
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
