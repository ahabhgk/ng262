use lexical::parse_float_options;
use num_bigint::BigInt;
use unicode_xid::UnicodeXID;

use super::{
  error::{SyntaxError, SyntaxErrorInfo, SyntaxErrorTemplate},
  source::Source,
  strict::{Strict, UseStrict},
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

pub struct Lexer<'i, 's> {
  source: Source<'i>,
  current_token: Option<Token>,
  peek_token: Option<Token>,
  peek_ahead_token: Option<Token>,
  line: usize,
  column_offset: usize,
  line_terminator_before_next_token: bool,
  had_escaped: bool,
  // TODO: use derive marco
  strict: &'s mut Strict,
}

impl UseStrict for Lexer<'_, '_> {
  fn is_strict(&self) -> bool {
    self.strict.is_strict()
  }

  fn use_strict(&mut self, is_strict: bool) {
    self.strict.use_strict(is_strict);
  }
}

impl SyntaxErrorInfo for Lexer<'_, '_> {
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

impl Lexer<'_, '_> {
  pub fn get_source(&self) -> &Source {
    &self.source
  }

  pub fn forward(&mut self) -> Result<(), SyntaxError> {
    self.current_token = Some(self.peek()?);
    self.peek_token = Some(self.peek_ahead()?);
    self.peek_ahead_token = None;
    Ok(())
  }

  pub fn current(&self) -> Token {
    self
      .current_token
      .clone()
      .expect("current() should not call before forward()")
  }

  #[allow(clippy::should_implement_trait)]
  pub fn next(&mut self) -> Result<Token, SyntaxError> {
    self.forward()?;
    Ok(self.current())
  }

  pub fn peek(&mut self) -> Result<Token, SyntaxError> {
    if self.peek_token.is_none() {
      self.peek_token = Some(self.advance()?);
    }
    Ok(self.peek_token.clone().unwrap())
  }

  pub fn peek_ahead(&mut self) -> Result<Token, SyntaxError> {
    if self.peek_token.is_none() {
      self.peek_token = Some(self.advance()?);
    }
    if self.peek_ahead_token.is_none() {
      self.peek_ahead_token = Some(self.advance()?);
    }
    Ok(self.peek_ahead_token.clone().unwrap())
  }

  pub fn matches(&self, token_type: TokenType, peek: Token) -> bool {
    peek.token_type == token_type
  }

  pub fn matches_identifier(&self, id: &str, peek: Token) -> bool {
    if matches!(peek.token_type, TokenType::IDENTIFIER(s) if s == id) {
      !self
        .source
        .slice(peek.start_index, peek.end_index)
        .contains('\\')
    } else {
      false
    }
  }

  pub fn test(&mut self, token_type: TokenType) -> Result<bool, SyntaxError> {
    let peek = self.peek()?;
    Ok(self.matches(token_type, peek))
  }

  pub fn test_identifier(&mut self, id: &str) -> Result<bool, SyntaxError> {
    let peek = self.peek()?;
    Ok(self.matches_identifier(id, peek))
  }

  pub fn test_ahead(
    &mut self,
    token_type: TokenType,
  ) -> Result<bool, SyntaxError> {
    let peek = self.peek_ahead()?;
    Ok(self.matches(token_type, peek))
  }

  pub fn test_ahead_identifier(
    &mut self,
    id: &str,
  ) -> Result<bool, SyntaxError> {
    let peek = self.peek_ahead()?;
    Ok(self.matches_identifier(id, peek))
  }

  pub fn eat(&mut self, token_type: TokenType) -> Result<bool, SyntaxError> {
    if self.test(token_type)? {
      self.forward()?;
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn eat_identifier(&mut self, id: &str) -> Result<bool, SyntaxError> {
    if self.test_identifier(id)? {
      self.forward()?;
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn expect(&mut self, token_type: TokenType) -> Result<(), SyntaxError> {
    if self.test(token_type)? {
      self.forward()?;
      Ok(())
    } else {
      Err(SyntaxError::from_index(
        self,
        0,
        SyntaxErrorTemplate::UnexpectedToken,
      ))
    }
  }

  pub fn expect_identifier(&mut self, id: &str) -> Result<(), SyntaxError> {
    if self.test_identifier(id)? {
      self.forward()?;
      Ok(())
    } else {
      Err(SyntaxError::from_index(
        self,
        0,
        SyntaxErrorTemplate::UnexpectedToken,
      ))
    }
  }
}

impl<'i, 's> Lexer<'i, 's> {
  pub fn new(s: &'i str, strict: &'s mut Strict) -> Self {
    Self {
      source: Source::new(s),
      current_token: None,
      peek_token: None,
      peek_ahead_token: None,
      line: 1,
      column_offset: 0,
      line_terminator_before_next_token: false,
      had_escaped: false,
      strict,
    }
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
    Token {
      token_type,
      start_index,
      end_index: self.source.index(),
      line,
      column,
      had_line_terminator_before: self.line_terminator_before_next_token,
      had_escaped: self.had_escaped,
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
          '?' => match self.source.next() {
            Some('.') => {
              if matches!(self.source.peek(), Some(c) if !is_decimal_digit(c)) {
                self.source.forward();
                Some(TokenType::OPTIONAL)
              } else {
                None
              }
            }
            Some('?') => match self.source.next() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::ASSIGN_NULLISH)
              }
              _ => Some(TokenType::NULLISH),
            },
            _ => Some(TokenType::CONDITIONAL),
          },
          // < <= << <<=
          '<' => match self.source.next() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::LTE)
            }
            Some('<') => match self.source.next() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::ASSIGN_SHL)
              }
              _ => Some(TokenType::SHL),
            },
            _ => Some(TokenType::LT),
          },
          // > >= >> >>= >>> >>>=
          '>' => match self.source.next() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::GTE)
            }
            Some('>') => match self.source.next() {
              Some('>') => match self.source.next() {
                Some('=') => {
                  self.source.forward();
                  Some(TokenType::ASSIGN_SHR)
                }
                _ => Some(TokenType::SHR),
              },
              Some('=') => {
                self.source.forward();
                Some(TokenType::ASSIGN_SAR)
              }
              _ => Some(TokenType::SAR),
            },
            _ => Some(TokenType::GT),
          },
          // = == === =>
          '=' => match self.source.next() {
            Some('=') => match self.source.next() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::EQ_STRICT)
              }
              _ => Some(TokenType::EQ),
            },
            Some('>') => {
              self.source.forward();
              Some(TokenType::ARROW)
            }
            _ => Some(TokenType::ASSIGN),
          },
          // ! != !==
          '!' => match self.source.next() {
            Some('=') => match self.source.next() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::NE_STRICT)
              }
              _ => Some(TokenType::NE),
            },
            _ => Some(TokenType::NOT),
          },
          // + ++ +=
          '+' => match self.source.next() {
            Some('+') => {
              self.source.forward();
              Some(TokenType::INC)
            }
            Some('=') => {
              self.source.forward();
              Some(TokenType::ASSIGN_ADD)
            }
            _ => Some(TokenType::ADD),
          },
          // - -- -=
          '-' => match self.source.next() {
            Some('-') => {
              self.source.forward();
              Some(TokenType::DEC)
            }
            Some('=') => {
              self.source.forward();
              Some(TokenType::ASSIGN_SUB)
            }
            _ => Some(TokenType::SUB),
          },
          // * *= ** **=
          '*' => match self.source.next() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::ASSIGN_MUL)
            }
            Some('*') => match self.source.next() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::ASSIGN_EXP)
              }
              _ => Some(TokenType::EXP),
            },
            _ => Some(TokenType::MUL),
          },
          // % %=
          '%' => match self.source.next() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::ASSIGN_MOD)
            }
            _ => Some(TokenType::MOD),
          },
          // / /=
          '/' => match self.source.next() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::ASSIGN_DIV)
            }
            _ => Some(TokenType::DIV),
          },
          // & && &= &&=
          '&' => match self.source.next() {
            Some('&') => match self.source.next() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::ASSIGN_AND)
              }
              _ => Some(TokenType::AND),
            },
            Some('=') => Some(TokenType::ASSIGN_BIT_AND),
            _ => Some(TokenType::BIT_AND),
          },
          // | || |= ||=
          '|' => match self.source.next() {
            Some('|') => match self.source.next() {
              Some('=') => {
                self.source.forward();
                Some(TokenType::ASSIGN_OR)
              }
              _ => Some(TokenType::OR),
            },
            Some('=') => {
              self.source.forward();
              Some(TokenType::ASSIGN_BIT_OR)
            }
            _ => Some(TokenType::BIT_OR),
          },
          // ^ ^=
          '^' => match self.source.next() {
            Some('=') => {
              self.source.forward();
              Some(TokenType::ASSIGN_BIT_XOR)
            }
            _ => Some(TokenType::BIT_XOR),
          },
          // . ... NUMBER
          '.' => match self.source.next() {
            Some('.') => {
              if let Some('.') = self.source.next() {
                self.source.forward();
                Some(TokenType::ELLIPSIS)
              } else {
                None
              }
            }
            Some(c) if is_decimal_digit(c) => {
              self.source.backward();
              Some(self.scan_number()?)
            }
            _ => Some(TokenType::PERIOD),
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
      Some(TokenType::EOS)
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
      match self.source.next() {
        Some('x' | 'X') => base = 16,
        Some('o' | 'O') => base = 8,
        Some('b' | 'B') => base = 2,
        Some('e' | 'E' | '.') => {}
        Some('n') => {
          self.source.forward();
          return Ok(TokenType::BIGINT(
            BigInt::parse_bytes(b"0", 10)
              .expect("failed to parse string as a bigint"),
          ));
        }
        _ => return Ok(TokenType::NUMBER(0.0)),
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
          return Ok(TokenType::NUMBER(0.0));
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
      return Ok(TokenType::BIGINT(
        BigInt::parse_bytes(buffer.as_bytes(), 10)
          .expect("failed to parse string as a bigint"),
      ));
    }
    // .
    if base == 10 && self.source.current() == Some('.') {
      if let Some('_') = self.source.next() {
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
    Ok(TokenType::NUMBER(num))
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

    Ok(TokenType::STRING(buffer))
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
        if matches!(self.source.next(), Some(c) if c != 'u') {
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
          Ok(TokenType::PRIVATE_IDENTIFIER(buffer))
        } else {
          Ok(TokenType::IDENTIFIER(buffer))
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
        } else if self.is_strict() && is_decimal_digit(c) {
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
    let position = self.source.index();
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
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::LBRACE,
      TokenType::RBRACE,
      TokenType::EOS,
    );
  }

  #[test]
  fn number_dot_dot() {
    let source = "123..toString()";
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::NUMBER(123.0),
      TokenType::PERIOD,
      TokenType::IDENTIFIER("toString".to_owned()),
      TokenType::LPAREN,
      TokenType::RPAREN,
      TokenType::EOS,
    );
  }

  #[test]
  fn identifier_async() {
    let source = r#"async"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::IDENTIFIER("async".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn identifier_escape_unicode() {
    let source = r#"a\u0061"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::IDENTIFIER("aa".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn identifier_escape_unicode_2() {
    let source = r#"℘\u2118"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::IDENTIFIER("℘℘".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn identifier_dollar() {
    let source = r#"$jq"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::IDENTIFIER("$jq".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn keyword_escape() {
    let source = r#"\u{61}wait"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::ESCAPED_KEYWORD("await".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn private_identifier_escape() {
    let source = r#"#a\u{61}pple"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::PRIVATE_IDENTIFIER("aapple".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn string_escape() {
    let source = r#"'\n'"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::STRING("\n".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn string_escape_2() {
    let source = r#"'\\n'"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::STRING("\\n".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn string_escape_hex() {
    let source = r#"'\x61'"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::STRING("a".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn string_escape_long_unicode() {
    let source = r#"'\u{00000000034}'"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::STRING("4".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn string_literal() {
    let source = r#"'ng262'"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::STRING("ng262".to_owned()),
      TokenType::EOS,
    );
  }

  #[test]
  fn number_literal() {
    let source = r#"123.0"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(lexer, TokenType::NUMBER(123.0), TokenType::EOS);
  }

  #[test]
  fn big_int_literal() {
    let source = r#"9007199254740993n"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(
      lexer,
      TokenType::BIGINT(BigInt::parse_bytes(b"9007199254740993", 10).unwrap()),
      TokenType::EOS
    );
  }

  #[test]
  fn number_exponent() {
    let source = r#"1e2"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(lexer, TokenType::NUMBER(100.0), TokenType::EOS);
  }

  #[test]
  fn number_signed_exponent() {
    let source = r#"1e-2"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(lexer, TokenType::NUMBER(0.01), TokenType::EOS);
  }

  #[test]
  fn number_hex() {
    let source = r#"0x000000000"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(lexer, TokenType::NUMBER(0.0), TokenType::EOS);
  }

  #[test]
  fn number_point() {
    let source = r#"1.123"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(lexer, TokenType::NUMBER(1.123), TokenType::EOS);
  }

  #[test]
  fn number_separator() {
    let source = r#"123_456_789"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_token_type!(lexer, TokenType::NUMBER(123_456_789.0), TokenType::EOS);
  }

  #[test]
  fn lexer_forward() {
    let source = r#"let ng = 262;"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    lexer.forward().unwrap();
    assert_eq!(
      lexer.current().token_type,
      TokenType::IDENTIFIER("let".to_owned())
    );
    lexer.forward().unwrap();
    assert_eq!(
      lexer.current().token_type,
      TokenType::IDENTIFIER("ng".to_owned())
    );
    lexer.forward().unwrap();
    assert_eq!(lexer.current().token_type, TokenType::ASSIGN);
    lexer.forward().unwrap();
    assert_eq!(lexer.current().token_type, TokenType::NUMBER(262.0));
    lexer.forward().unwrap();
    assert_eq!(lexer.current().token_type, TokenType::SEMICOLON);
    lexer.forward().unwrap();
    assert_eq!(lexer.current().token_type, TokenType::EOS);
  }

  #[test]
  fn lexer_peek_at_start() {
    let source = r#"let ng = 262;"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_eq!(
      lexer.peek().unwrap().token_type,
      TokenType::IDENTIFIER("let".to_owned())
    );
    assert_eq!(
      lexer.peek_ahead().unwrap().token_type,
      TokenType::IDENTIFIER("ng".to_owned())
    );
  }

  #[test]
  fn lexer_peek_at_end() {
    let source = r#";"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert_eq!(lexer.next().unwrap().token_type, TokenType::SEMICOLON);
    assert_eq!(lexer.peek().unwrap().token_type, TokenType::EOS);
    assert_eq!(lexer.peek_ahead().unwrap().token_type, TokenType::EOS);
  }

  #[test]
  fn lexer_matches() {
    let source = r#";"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    let peek = lexer.peek().unwrap();
    assert!(lexer.matches(TokenType::SEMICOLON, peek));
  }

  #[test]
  fn lexer_matches_identifier() {
    let source = r#"let"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    let peek = lexer.peek().unwrap();
    assert!(lexer.matches_identifier("let", peek));
  }

  #[test]
  fn lexer_test() {
    let source = r#";;"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert!(lexer.test(TokenType::SEMICOLON).unwrap());
    assert!(lexer.test_ahead(TokenType::SEMICOLON).unwrap());
  }

  #[test]
  fn lexer_test_identifier() {
    let source = r#"async async"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert!(lexer.test_identifier("async").unwrap());
    assert!(lexer.test_ahead_identifier("async").unwrap());
  }

  #[test]
  fn lexer_eat() {
    let source = r#";"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert!(lexer.eat(TokenType::SEMICOLON).unwrap());
    assert!(lexer.matches(TokenType::SEMICOLON, lexer.current()));
    assert!(lexer.eat(TokenType::EOS).unwrap());
  }

  #[test]
  fn lexer_eat_identifier() {
    let source = r#"async"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert!(lexer.eat_identifier("async").unwrap());
    assert!(
      lexer.matches(TokenType::IDENTIFIER("async".to_owned()), lexer.current())
    );
    assert!(lexer.test(TokenType::EOS).unwrap());
  }

  #[test]
  fn lexer_expect() {
    let source = r#";"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert!(lexer.expect(TokenType::SEMICOLON).is_ok());
    assert!(lexer.expect(TokenType::SEMICOLON).is_err());
    assert!(lexer.expect(TokenType::EOS).is_ok());
  }

  #[test]
  fn lexer_expect_identifier() {
    let source = r#"async"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    assert!(lexer.expect_identifier("async").is_ok());
    assert!(lexer.expect_identifier("async").is_err());
    assert!(lexer.expect(TokenType::EOS).is_ok());
  }

  #[test]
  fn lexer_next() {
    let source = r#"async;"#;
    let strict = &mut Strict::new(false);
    let mut lexer = Lexer::new(source, strict);
    let peek = lexer.peek().unwrap();
    assert!(lexer.matches(TokenType::IDENTIFIER("async".to_owned()), peek));
    let next = lexer.next().unwrap();
    assert!(lexer.matches(TokenType::IDENTIFIER("async".to_owned()), next));
    let next = lexer.next().unwrap();
    assert!(lexer.matches(TokenType::SEMICOLON, next));
    let next = lexer.next().unwrap();
    assert!(lexer.matches(TokenType::EOS, next));
  }
}
