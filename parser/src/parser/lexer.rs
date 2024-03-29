use lexical::parse_float_options;
use num_bigint::BigInt;
use unicode_xid::UnicodeXID;

use super::{
  error::{SyntaxError, SyntaxErrorInfo, SyntaxErrorTemplate},
  source::Source,
  tokens::{lookup_keyword, Token, TokenType},
};

pub fn is_line_terminator(c: char) -> bool {
  matches!(c, '\r' | '\n' | '\u{2028}' | '\u{2029}')
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

fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

fn is_octal_digit(c: char) -> bool {
  c.is_digit(8)
}

fn is_binary_digit(c: char) -> bool {
  c.is_digit(2)
}

fn is_identifier_start(c: char) -> bool {
  c.is_ascii_alphanumeric()
    || c == '$'
    || c == '_'
    || c == '\\'
    || c.is_xid_start()
}

fn is_identifier_part(c: char) -> bool {
  c.is_ascii_alphanumeric()
    || c == '$'
    || c == '_'
    || c == '\\'
    || c == '\u{200C}'
    || c == '\u{200D}'
    || c.is_xid_continue()
}

fn is_lead_surrogate(cp: char) -> bool {
  cp >= unsafe { char::from_u32_unchecked(0xD800) }
    && cp <= unsafe { char::from_u32_unchecked(0xDBFF) }
}

fn is_trail_surrogate(cp: char) -> bool {
  cp >= unsafe { char::from_u32_unchecked(0xDC00) }
    && cp <= unsafe { char::from_u32_unchecked(0xDFFF) }
}

pub struct Lexer {
  source: Source,
  // start
  line: usize,
  column_offset: usize,
  line_terminator_before_next_token: bool,
  had_escaped: bool,
  is_strict: bool,
  // iter
  current_token: Option<Token>,
  peek_token: Option<Token>,
  peek_ahead_token: Option<Token>,
}

impl SyntaxErrorInfo for Lexer {
  fn index(&self) -> usize {
    self.source.index()
  }

  fn line(&self) -> usize {
    self.line
  }

  fn get(&self, cursor: usize) -> Option<char> {
    self.source.get(cursor)
  }

  fn slice(&self, start_cursor: usize, end_cursor: usize) -> String {
    self.source.slice(start_cursor, end_cursor)
  }
}

impl Lexer {
  pub fn new(s: &'static str, is_strict: bool) -> Self {
    Self {
      source: Source::new(s),
      line: 1,
      column_offset: 0,
      line_terminator_before_next_token: false,
      had_escaped: false,
      is_strict,
      current_token: None,
      peek_token: None,
      peek_ahead_token: None,
    }
  }

  pub fn get_source(&self) -> &Source {
    &self.source
  }

  pub fn forward(&mut self) -> Result<(), SyntaxError> {
    self.current_token = Some(self.peek()?.to_owned());
    self.peek_token = Some(self.peek_ahead()?.to_owned());
    self.peek_ahead_token = None;
    Ok(())
  }

  pub fn current(&self) -> &Token {
    self
      .current_token
      .as_ref()
      .expect("current() should not call before forward()")
  }

  pub fn bump(&mut self) -> Result<&Token, SyntaxError> {
    self.forward()?;
    Ok(self.current())
  }

  pub fn peek(&mut self) -> Result<&Token, SyntaxError> {
    if self.peek_token.is_none() {
      self.peek_token = Some(self.advance()?);
    }
    Ok(self.peek_token.as_ref().unwrap())
  }

  pub fn peek_ahead(&mut self) -> Result<&Token, SyntaxError> {
    if self.peek_token.is_none() {
      self.peek_token = Some(self.advance()?);
    }
    if self.peek_ahead_token.is_none() {
      self.peek_ahead_token = Some(self.advance()?);
    }
    Ok(self.peek_ahead_token.as_ref().unwrap())
  }

  fn advance(&mut self) -> Result<Token, SyntaxError> {
    self.line_terminator_before_next_token = false;
    self.had_escaped = false;
    self.next_token()
  }

  fn create_token(
    &self,
    token_type: TokenType,
    start_index: usize,
    line: usize,
    column: usize,
  ) -> Token {
    let end_index = self.source.index();
    Token {
      token_type,
      start_index,
      end_index,
      line,
      column,
      had_line_terminator_before: self.line_terminator_before_next_token,
      had_escaped: self.had_escaped,
      source_text: self.source.slice(start_index, end_index),
    }
  }

  fn next_token(&mut self) -> Result<Token, SyntaxError> {
    self.skip_space()?;

    // set token location info after skipping space
    let start_index = self.source.index();
    let line = self.line;
    let column = start_index - self.column_offset + 1;

    let token_type = if let Some(c) = self.source.current() {
      if c < char::from(127) {
        // fast path for usual case
        match c {
          '(' | ')' | '{' | '}' | '[' | ']' | ':' | ';' | ',' | '~' | '`' => {
            self.source.forward();
            Some(TokenType::from_single(c))
          }
          // ? ?. ?? ??=
          '?' => match self.source.bump() {
            Some('.') => {
              if matches!(self.source.peek(), Some(c) if !is_decimal_digit(c)) {
                self.source.forward();
                Some(TokenType::Optional)
              } else {
                None
              }
            }
            Some('?') => match self.source.bump() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::AssignNullish)
              }
              _ => Some(TokenType::Nullish),
            },
            _ => Some(TokenType::Conditional),
          },
          // < <= << <<=
          '<' => match self.source.bump() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::LessThanEqual)
            }
            Some('<') => match self.source.bump() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::AssignShl)
              }
              _ => Some(TokenType::Shl),
            },
            _ => Some(TokenType::LessThan),
          },
          // > >= >> >>= >>> >>>=
          '>' => match self.source.bump() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::GreaterThanEqual)
            }
            Some('>') => match self.source.bump() {
              Some('>') => match self.source.bump() {
                Some('=') => {
                  self.source.forward();
                  Some(TokenType::AssignShr)
                }
                _ => Some(TokenType::Shr),
              },
              Some('=') => {
                self.source.forward();
                Some(TokenType::AssignSar)
              }
              _ => Some(TokenType::Sar),
            },
            _ => Some(TokenType::GreaterThan),
          },
          // = == === =>
          '=' => match self.source.bump() {
            Some('=') => match self.source.bump() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::StrictEqual)
              }
              _ => Some(TokenType::Equal),
            },
            Some('>') => {
              self.source.forward();
              Some(TokenType::Arrow)
            }
            _ => Some(TokenType::Assign),
          },
          // ! != !==
          '!' => match self.source.bump() {
            Some('=') => match self.source.bump() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::StrictNotEqual)
              }
              _ => Some(TokenType::NotEqual),
            },
            _ => Some(TokenType::Not),
          },
          // + ++ +=
          '+' => match self.source.bump() {
            Some('+') => {
              self.source.forward();
              Some(TokenType::Inc)
            }
            Some('=') => {
              self.source.forward();
              Some(TokenType::AssignAdd)
            }
            _ => Some(TokenType::Add),
          },
          // - -- -=
          '-' => match self.source.bump() {
            Some('-') => {
              self.source.forward();
              Some(TokenType::Dec)
            }
            Some('=') => {
              self.source.forward();
              Some(TokenType::AssignSub)
            }
            _ => Some(TokenType::Sub),
          },
          // * *= ** **=
          '*' => match self.source.bump() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::AssignMul)
            }
            Some('*') => match self.source.bump() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::AssignExp)
              }
              _ => Some(TokenType::Exp),
            },
            _ => Some(TokenType::Mul),
          },
          // % %=
          '%' => match self.source.bump() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::AssignMod)
            }
            _ => Some(TokenType::Mod),
          },
          // / /=
          '/' => match self.source.bump() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::AssignDiv)
            }
            _ => Some(TokenType::Div),
          },
          // & && &= &&=
          '&' => match self.source.bump() {
            Some('&') => match self.source.bump() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::AssignAnd)
              }
              _ => Some(TokenType::And),
            },
            Some('=') => Some(TokenType::AssignBitAnd),
            _ => Some(TokenType::BitAnd),
          },
          // | || |= ||=
          '|' => match self.source.bump() {
            Some('|') => match self.source.bump() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::AssignOr)
              }
              _ => Some(TokenType::Or),
            },
            Some('=') => {
              self.source.forward();
              Some(TokenType::AssignBitOr)
            }
            _ => Some(TokenType::BitOr),
          },
          // ^ ^=
          '^' => match self.source.bump() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::AssignBitXor)
            }
            _ => Some(TokenType::BitXor),
          },
          // . ... NUMBER
          '.' => match self.source.bump() {
            Some('.') => {
              if let Some('.') = self.source.bump() {
                self.source.forward();
                Some(TokenType::Ellipsis)
              } else {
                None
              }
            }
            Some(c) if is_decimal_digit(c) => {
              self.source.backward();
              Some(self.scan_number()?)
            }
            _ => Some(TokenType::Period),
          },
          '"' | '\'' => {
            self.source.forward();
            Some(self.scan_string(c)?)
          }
          '0'..='9' => Some(self.scan_number()?),
          'a'..='z' | 'A'..='Z' | '$' | '_' | '\\' => {
            Some(self.scan_identifier_or_keyword(false)?)
          }
          '#' => {
            self.source.forward();
            Some(self.scan_identifier_or_keyword(true)?)
          }
          _ => None,
        }
      } else if is_lead_surrogate(c) || is_identifier_start(c) {
        Some(self.scan_identifier_or_keyword(false)?)
      } else {
        None
      }
    } else {
      Some(TokenType::EndOfSource)
    };

    match token_type {
      Some(t) => Ok(self.create_token(t, start_index, line, column)),
      None => Err(SyntaxError::from_index(
        self,
        0,
        SyntaxErrorTemplate::UnexpectedToken,
      )),
    }
  }

  /// See https://tc39.es/ecma262/#sec-literals-numeric-literals
  fn scan_number(&mut self) -> Result<TokenType, SyntaxError> {
    let start = self.source.index();
    let mut base = 10;
    let mut check: fn(char) -> bool = is_decimal_digit;
    // base
    if self.source.current() == Some('0') {
      match self.source.bump() {
        Some('x' | 'X') => base = 16,
        Some('o' | 'O') => base = 8,
        Some('b' | 'B') => base = 2,
        Some('e' | 'E' | '.') => {}
        Some('n') => {
          self.source.forward();
          return Ok(TokenType::BigInt(
            BigInt::parse_bytes(b"0", 10)
              .expect("failed to parse string as a bigint"),
          ));
        }
        _ => return Ok(TokenType::Number(0.0)),
      }
      check = match base {
        16 => is_hex_digit,
        10 => is_decimal_digit,
        8 => is_octal_digit,
        2 => is_binary_digit,
        _ => unreachable!("base is not correct when scan_number"),
      };
      if base != 10 {
        if matches!(self.source.peek(), Some(c) if !check(c)) {
          return Ok(TokenType::Number(0.0));
        }
        self.source.forward();
      }
    }
    // scan
    macro_rules! scan {
      () => {{
        while let Some(c) = self.source.current() {
          if check(c) {
            self.source.forward();
          } else if c == '_' {
            if matches!(self.source.peek(), Some(p) if !check(p)) {
              return Err(
                SyntaxError::from_index(
                  self,
                  1,
                  SyntaxErrorTemplate::UnexpectedToken,
                ));
            }
            self.source.forward();
          } else {
            break;
          }
        }
      }}
    }
    scan!();
    // n
    if self.source.current() == Some('n') {
      let buffer = self
        .source
        .slice(start, self.source.index())
        .replace('_', "");
      self.source.forward();
      return Ok(TokenType::BigInt(
        BigInt::parse_bytes(buffer.as_bytes(), 10)
          .expect("failed to parse string as a bigint"),
      ));
    }
    // .
    if base == 10 && self.source.current() == Some('.') {
      if let Some('_') = self.source.bump() {
        return Err(SyntaxError::from_index(
          self,
          0,
          SyntaxErrorTemplate::UnexpectedToken,
        ));
      }
      scan!();
    }
    // e E
    if base == 10
      && (self.source.current() == Some('e')
        || self.source.current() == Some('E'))
    {
      self.source.forward();
      if let Some('_') = self.source.current() {
        return Err(SyntaxError::from_index(
          self,
          0,
          SyntaxErrorTemplate::UnexpectedToken,
        ));
      }
      if let Some('-' | '+') = self.source.current() {
        self.source.forward();
      }
      if let Some('_') = self.source.current() {
        return Err(SyntaxError::from_index(
          self,
          0,
          SyntaxErrorTemplate::UnexpectedToken,
        ));
      }
      scan!();
    }

    if matches!(self.source.current(), Some(c) if is_identifier_start(c)) {
      return Err(SyntaxError::from_index(
        self,
        0,
        SyntaxErrorTemplate::UnexpectedToken,
      ));
    }
    // parse
    let buffer = self
      .source
      .slice(
        if base == 10 { start } else { start + 2 },
        self.source.index(),
      )
      .replace('_', "");
    const FORMAT: u128 = lexical::format::JAVASCRIPT_STRING;
    let num = lexical::parse_with_options::<f64, _, FORMAT>(
      buffer,
      &parse_float_options::JAVASCRIPT_STRING,
    )
    .expect("failed to parse string as a js number");
    Ok(TokenType::Number(num))
  }

  /// See https://tc39.es/ecma262/#sec-literals-string-literals
  fn scan_string(&mut self, quote: char) -> Result<TokenType, SyntaxError> {
    let mut buffer = String::new();
    loop {
      match self.source.current() {
        None => {
          return Err(SyntaxError::from_index(
            self,
            0,
            SyntaxErrorTemplate::UnterminatedString,
          ))
        }
        Some(c) => {
          if c == quote {
            self.source.forward();
            break;
          }
          if c == '\r' || c == '\n' {
            return Err(SyntaxError::from_index(
              self,
              0,
              SyntaxErrorTemplate::UnterminatedString,
            ));
          }
          self.source.forward();
          if c == '\\' {
            match self.source.current() {
              None => {
                return Err(SyntaxError::from_index(
                  self,
                  0,
                  SyntaxErrorTemplate::UnterminatedString,
                ))
              }
              Some(p) => {
                if is_line_terminator(p) {
                  self.terminate_line(p)
                } else {
                  buffer.push(self.scan_escape_sequence()?)
                }
              }
            }
          } else {
            buffer.push(c);
          }
        }
      }
    }

    Ok(TokenType::String(buffer))
  }

  /// See https://tc39.es/ecma262/#sec-names-and-keywords
  fn scan_identifier_or_keyword(
    &mut self,
    is_private: bool,
  ) -> Result<TokenType, SyntaxError> {
    let mut buffer = String::new();
    let mut had_escaped = false;
    let mut check: fn(char) -> bool = is_identifier_start;
    while let Some(c) = self.source.current() {
      if c == '\\' {
        if !had_escaped {
          had_escaped = true;
        }
        if matches!(self.source.bump(), Some(c) if c != 'u') {
          return Err(SyntaxError::from_index(
            self,
            0,
            SyntaxErrorTemplate::InvalidUnicodeEscape,
          ));
        }
        self.source.forward();
        let raw = char::from_u32(self.scan_code_point()?).unwrap();
        if !check(c) {
          return Err(SyntaxError::from_index(
            self,
            0,
            SyntaxErrorTemplate::InvalidUnicodeEscape,
          ));
        }
        buffer.push(raw)
      } else if is_lead_surrogate(c) {
        todo!("CombineSurrogatePair is not supported yet")
      } else if check(c) {
        buffer.push(c);
        self.source.forward();
      } else {
        break;
      }

      check = is_identifier_part;
    }

    match lookup_keyword(&buffer, had_escaped) {
      Some(t) if !is_private => Ok(t),
      _ => {
        self.had_escaped = had_escaped;
        if is_private {
          Ok(TokenType::PrivateIdentifier(buffer))
        } else {
          Ok(TokenType::Identifier(buffer))
        }
      }
    }
  }

  fn scan_code_point(&mut self) -> Result<u32, SyntaxError> {
    if let Some('{') = self.source.current() {
      match self.source.index_of('}') {
        Some(end) => {
          self.source.forward();
          let code = self.scan_hex(end - self.source.index())?;
          self.source.forward();
          if code > 0x10FFFF {
            Err(SyntaxError::from_index(
              self,
              0,
              SyntaxErrorTemplate::InvalidCodePoint,
            ))
          } else {
            Ok(code)
          }
        }
        None => Err(SyntaxError::from_index(
          self,
          0,
          SyntaxErrorTemplate::InvalidUnicodeEscape,
        )),
      }
    } else {
      self.scan_hex(4)
    }
  }

  fn scan_hex(&mut self, len: usize) -> Result<u32, SyntaxError> {
    if len == 0 {
      return Err(SyntaxError::from_index(
        self,
        0,
        SyntaxErrorTemplate::InvalidCodePoint,
      ));
    }
    let mut n = 0;
    for _ in 0..len {
      match self.source.current() {
        Some(c) if is_hex_digit(c) => {
          self.source.forward();
          n = (n << 4) | c.to_digit(16).unwrap();
        }
        _ => {
          return Err(SyntaxError::from_index(
            self,
            0,
            SyntaxErrorTemplate::UnexpectedToken,
          ))
        }
      }
    }
    Ok(n)
  }

  fn scan_escape_sequence(&mut self) -> Result<char, SyntaxError> {
    // unwrap: only used by scan_string when `self.source.current()` is not None
    match self.source.current().unwrap() {
      'b' => {
        self.source.forward();
        Ok('\u{0008}')
      }
      't' => {
        self.source.forward();
        Ok('\t')
      }
      'n' => {
        self.source.forward();
        Ok('\n')
      }
      'v' => {
        self.source.forward();
        Ok('\u{000b}')
      }
      'f' => {
        self.source.forward();
        Ok('\u{000c}')
      }
      'r' => {
        self.source.forward();
        Ok('\r')
      }
      'x' => {
        self.source.forward();
        Ok(char::from_u32(self.scan_hex(2)?).unwrap())
      }
      'u' => {
        self.source.forward();
        Ok(char::from_u32(self.scan_code_point()?).unwrap())
      }
      c => {
        if c == '0'
          && matches!(self.source.peek(), Some(p) if is_decimal_digit(p))
        {
          self.source.forward();
          Ok('\u{0000}')
        } else if self.is_strict && is_decimal_digit(c) {
          Err(SyntaxError::from_index(
            self,
            0,
            SyntaxErrorTemplate::IllegalOctalEscape,
          ))
        } else {
          self.source.forward();
          Ok(c)
        }
      }
    }
  }

  fn skip_hashbang_comment(&mut self) {
    if self.source.index() == 0
      && matches!(self.source.current(), Some('#'))
      && matches!(self.source.peek(), Some('!'))
    {
      self.skip_line_comment();
    }
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
          if is_line_terminator(c) {
            self.terminate_line(c);
          } else if is_whitespace(c) {
            self.source.forward();
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
    self.column_offset = self.source.index();
    self.line_terminator_before_next_token = true;
  }

  fn skip_line_comment(&mut self) {
    self.source.forward();
    self.source.forward();
    while let Some(c) = self.source.current() {
      if is_line_terminator(c) {
        self.terminate_line(c);
        break;
      } else {
        self.source.forward();
      }
    }
  }

  fn skip_block_comment(&mut self) -> Result<(), SyntaxError> {
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
          return Err(SyntaxError::from_index(
            self,
            0,
            SyntaxErrorTemplate::UnterminatedComment,
          ))
        }
      }
    }
    Ok(())
  }

  // fn unexpected(&self) -> SyntaxError {
  //   return self.create_syntax_error(
  //     self.source.position(),
  //     SyntaxErrorTemplate::UnexpectedToken,
  //   );
  // }
}

#[macro_export]
macro_rules! matches_token_type {
  ($peek:expr, $id:literal) => {{
    use $crate::parser::source::SourceText;
    use $crate::parser::tokens::TokenType;
    let peek = $peek;
    $crate::matches_token_type!(peek, TokenType::Identifier(s) if s == $id && !peek.source_text().contains('\\'))
  }};
  ($peek:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
    matches!(&$peek.token_type, $( $pattern )|+ $( if $guard )?)
  };
}

#[macro_export]
macro_rules! test {
  ($lexer:expr, $id:literal) => {
    $lexer.peek().map(|peek| $crate::matches_token_type!(peek, $id))
  };
  ($lexer:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
    $lexer.peek().map(|peek| $crate::matches_token_type!(peek, $( $pattern )|+ $( if $guard )?))
  };
}

#[macro_export]
macro_rules! test_ahead {
  ($lexer:expr, $id:literal) => {
    $lexer.peek_ahead().map(|peek| $crate::matches_token_type!(peek, $id))
  };
  ($lexer:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
    $lexer.peek_ahead().map(|peek| $crate::matches_token_type!(peek, $( $pattern )|+ $( if $guard )?))
  };
}

#[macro_export]
macro_rules! eat {
  ($lexer:expr, $id:literal) => {{
    let lexer = $lexer;
    $crate::test!(lexer, $id).and_then(|res| match res {
      true => lexer.forward().map(|_| true),
      false => Ok(false),
    })
  }};
  ($lexer:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {{
    let lexer = $lexer;
    $crate::test!(lexer, $( $pattern )|+ $( if $guard )?).and_then(|res| match res {
      true => lexer.forward().map(|_| true),
      false => Ok(false),
    })
  }};
}

#[macro_export]
macro_rules! expect {
  ($lexer:expr, $id:literal) => {{
    use $crate::parser::error::{SyntaxError, SyntaxErrorTemplate};
    let lexer = $lexer;
    $crate::test!(lexer, $id).and_then(|res| match res {
      true => lexer.bump(),
      false => match lexer.peek() {
        Ok(peek) => {
          let peek = peek.to_owned();
          Err(SyntaxError::from_token(
            lexer,
            &peek,
            SyntaxErrorTemplate::UnexpectedToken,
          ))
        },
        Err(e) => Err(e),
      },
    })
  }};
  ($lexer:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {{
    use $crate::parser::error::{SyntaxError, SyntaxErrorTemplate};
    let lexer = $lexer;
    $crate::test!(lexer, $( $pattern )|+ $( if $guard )?).and_then(|res| match res {
      true => lexer.bump(),
      false => match lexer.peek() {
        Ok(peek) => {
          let peek = peek.to_owned();
          Err(SyntaxError::from_token(
            lexer,
            &peek,
            SyntaxErrorTemplate::UnexpectedToken,
          ))
        },
        Err(e) => Err(e),
      },
    })
  }};
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! assert_token_type {
    ($l: ident, $t: expr) => {{
      let token_type = $t;
      let expected = $l.advance().unwrap().token_type;
      assert_eq!(expected, token_type);
    }};
    ($l: ident, $($t: expr),* $(,)?) => {{
      let ts = vec![$($t),*];
      for t in ts {
        assert_token_type!($l, t);
      }
    }}
  }

  #[test]
  fn comments() {
    let source = r#"/*
block comment
*/{
// line comment
}
"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::LBrace,
      TokenType::RBrace,
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn number_dot_dot() {
    let source = "123..toString()";
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::Number(123.0),
      TokenType::Period,
      TokenType::Identifier("toString".to_owned()),
      TokenType::LParen,
      TokenType::RParen,
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn identifier_async() {
    let source = r#"async"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::Identifier("async".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn identifier_escape_unicode() {
    let source = r#"a\u0061"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::Identifier("aa".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn identifier_escape_unicode_2() {
    let source = r#"℘\u2118"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::Identifier("℘℘".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn identifier_dollar() {
    let source = r#"$jq"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::Identifier("$jq".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn keyword_escape() {
    let source = r#"\u{61}wait"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::EscapedKeyword("await".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn private_identifier_escape() {
    let source = r#"#a\u{61}pple"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::PrivateIdentifier("aapple".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn string_escape() {
    let source = r#"'\n'"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::String("\n".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn string_escape_2() {
    let source = r#"'\\n'"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::String("\\n".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn string_escape_hex() {
    let source = r#"'\x61'"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::String("a".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn string_escape_long_unicode() {
    let source = r#"'\u{00000000034}'"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::String("4".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn string_literal() {
    let source = r#"'ng262'"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::String("ng262".to_owned()),
      TokenType::EndOfSource,
    );
  }

  #[test]
  fn number_literal() {
    let source = r#"123.0"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(lexer, TokenType::Number(123.0), TokenType::EndOfSource);
  }

  #[test]
  fn big_int_literal() {
    let source = r#"9007199254740993n"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::BigInt(BigInt::parse_bytes(b"9007199254740993", 10).unwrap()),
      TokenType::EndOfSource
    );
  }

  #[test]
  fn number_exponent() {
    let source = r#"1e2"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(lexer, TokenType::Number(100.0), TokenType::EndOfSource);
  }

  #[test]
  fn number_signed_exponent() {
    let source = r#"1e-2"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(lexer, TokenType::Number(0.01), TokenType::EndOfSource);
  }

  #[test]
  fn number_hex() {
    let source = r#"0x000000000"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(lexer, TokenType::Number(0.0), TokenType::EndOfSource);
  }

  #[test]
  fn number_point() {
    let source = r#"1.123"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(lexer, TokenType::Number(1.123), TokenType::EndOfSource);
  }

  #[test]
  fn number_separator() {
    let source = r#"123_456_789"#;
    let mut lexer = Lexer::new(source, false);
    assert_token_type!(
      lexer,
      TokenType::Number(123_456_789.0),
      TokenType::EndOfSource
    );
  }

  #[test]
  fn lexer_forward() {
    let source = r#"let ng = 262;"#;
    let mut lexer = Lexer::new(source, false);
    lexer.forward().unwrap();
    assert_eq!(
      lexer.current().token_type,
      TokenType::Identifier("let".to_owned())
    );
    lexer.forward().unwrap();
    assert_eq!(
      lexer.current().token_type,
      TokenType::Identifier("ng".to_owned())
    );
    lexer.forward().unwrap();
    assert_eq!(lexer.current().token_type, TokenType::Assign);
    lexer.forward().unwrap();
    assert_eq!(lexer.current().token_type, TokenType::Number(262.0));
    lexer.forward().unwrap();
    assert_eq!(lexer.current().token_type, TokenType::Semicolon);
    lexer.forward().unwrap();
    assert_eq!(lexer.current().token_type, TokenType::EndOfSource);
  }

  #[test]
  fn lexer_peek_at_start() {
    let source = r#"let ng = 262;"#;
    let mut lexer = Lexer::new(source, false);
    assert_eq!(
      lexer.peek().unwrap().token_type,
      TokenType::Identifier("let".to_owned())
    );
    assert_eq!(
      lexer.peek_ahead().unwrap().token_type,
      TokenType::Identifier("ng".to_owned())
    );
  }

  #[test]
  fn lexer_peek_at_end() {
    let source = r#";"#;
    let mut lexer = Lexer::new(source, false);
    assert_eq!(lexer.bump().unwrap().token_type, TokenType::Semicolon);
    assert_eq!(lexer.peek().unwrap().token_type, TokenType::EndOfSource);
    assert_eq!(
      lexer.peek_ahead().unwrap().token_type,
      TokenType::EndOfSource
    );
  }

  #[test]
  fn lexer_matches() {
    let source = r#";"#;
    let mut lexer = Lexer::new(source, false);
    let peek = lexer.peek().unwrap();
    assert!(matches_token_type!(peek, TokenType::Semicolon));
  }

  #[test]
  fn lexer_matches_identifier() {
    let source = r#"let"#;
    let mut lexer = Lexer::new(source, false);
    let peek = lexer.peek().unwrap();
    assert!(matches_token_type!(peek, "let"));
  }

  #[test]
  fn lexer_test() {
    let source = r#";;"#;
    let mut lexer = Lexer::new(source, false);
    assert!(test!(lexer, TokenType::Semicolon).unwrap());
    assert!(test_ahead!(lexer, TokenType::Semicolon).unwrap());
  }

  #[test]
  fn lexer_test_identifier() {
    let source = r#"async async"#;
    let mut lexer = Lexer::new(source, false);
    assert!(test!(lexer, "async").unwrap());
    assert!(test_ahead!(lexer, "async").unwrap());
  }

  #[test]
  fn lexer_eat() {
    let source = r#";"#;
    let mut lexer = Lexer::new(source, false);
    assert!(eat!(&mut lexer, TokenType::Semicolon).unwrap());
    assert!(matches_token_type!(lexer.current(), TokenType::Semicolon));
    assert!(eat!(&mut lexer, TokenType::EndOfSource).unwrap());
  }

  #[test]
  fn lexer_eat_identifier() {
    let source = r#"async"#;
    let mut lexer = Lexer::new(source, false);
    assert!(eat!(&mut lexer, "async").unwrap());
    assert!(matches_token_type!(lexer.current(), "async"));
    assert!(test!(lexer, TokenType::EndOfSource).unwrap());
  }

  #[test]
  fn lexer_expect() {
    let source = r#";"#;
    let mut lexer = Lexer::new(source, false);
    assert!(expect!(&mut lexer, TokenType::Semicolon).is_ok());
    assert!(expect!(&mut lexer, TokenType::Semicolon).is_err());
    assert!(expect!(&mut lexer, TokenType::EndOfSource).is_ok());
  }

  #[test]
  fn lexer_expect_identifier() {
    let source = r#"async"#;
    let mut lexer = Lexer::new(source, false);
    assert!(expect!(&mut lexer, "async").is_ok());
    assert!(expect!(&mut lexer, "async").is_err());
    assert!(expect!(&mut lexer, TokenType::EndOfSource).is_ok());
  }

  #[test]
  fn lexer_next() {
    let source = r#"async;"#;
    let mut lexer = Lexer::new(source, false);
    let peek = lexer.peek().unwrap();
    assert!(matches_token_type!(peek, "async"));
    let next = lexer.bump().unwrap();
    assert!(matches_token_type!(next, "async"));
    let next = lexer.bump().unwrap();
    assert!(matches_token_type!(next, TokenType::Semicolon));
    let next = lexer.bump().unwrap();
    assert!(matches_token_type!(next, TokenType::EndOfSource));
  }
}
