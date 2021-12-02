#[allow(non_camel_case_types)]
pub enum TokenType {
  // BEGIN PropertyOrCall
  // BEGIN Member
  // BEGIN Template
  TEMPLATE, // `
  // END Template

  // BEGIN Property
  PERIOD, // .
  LBRACK, // [
  // END Property
  // END Member
  OPTIONAL, // ?.
  LPAREN,   // (
  // END PropertyOrCall
  RPAREN,      // )
  RBRACK,      // ]
  LBRACE,      // {
  COLON,       // :
  ELLIPSIS,    // ...
  CONDITIONAL, // ?
  // BEGIN AutoSemicolon
  SEMICOLON, // ;
  RBRACE,    // }

  EOS,
  // END AutoSemicolon

  // BEGIN ArrowOrAssign
  ARROW, // =>
  // BEGIN Assign
  ASSIGN, // =
  // Logical
  ASSIGN_NULLISH, // ??=
  ASSIGN_OR,      // ||=
  ASSIGN_AND,     // &&=

  // Binop
  ASSIGN_BIT_OR,  // |=
  ASSIGN_BIT_XOR, // ^=
  ASSIGN_BIT_AND, // &=
  ASSIGN_SHL,     // <<=
  ASSIGN_SAR,     // >>=
  ASSIGN_SHR,     // >>>=
  ASSIGN_MUL,     // *=
  ASSIGN_DIV,     // /=
  ASSIGN_MOD,     // %=
  ASSIGN_EXP,     // **=

  // Unop
  ASSIGN_ADD, // +=
  ASSIGN_SUB, // -=
  // END Assign
  // END ArrowOrAssign

  // Binary operators by precidence
  COMMA, // ,

  // Logical
  NULLISH, // ??
  OR,      // ||
  AND,     // &&

  // Binop
  BIT_OR,  // |
  BIT_XOR, // ^
  BIT_AND, // &
  SHL,     // <<
  SAR,     // >>
  SHR,     // >>>
  MUL,     // *
  DIV,     // /
  MOD,     // %
  EXP,     // **

  // Unop
  ADD, // +
  SUB, // -

  NOT,     // !
  BIT_NOT, // ~
  DELETE,  // delete
  TYPEOF,  // typeof
  VOID,    // void

  // BEGIN IsCountOp
  INC, // ++
  DEC, // --
  // END IsCountOp
  // END IsUnaryOrCountOp
  EQ,         // ==
  EQ_STRICT,  // ===
  NE,         // !=
  NE_STRICT,  // !==
  LT,         // <
  GT,         // >
  LTE,        // <=
  GTE,        // >=
  INSTANCEOF, // instanceof
  IN,         // in

  BREAK,    // break
  CASE,     // case
  CATCH,    // catch
  CONTINUE, // continue
  DEBUGGER, // debugger
  DEFAULT,  // default
  // DELETE
  DO,       // do
  ELSE,     // else
  FINALLY,  // finally
  FOR,      // for
  FUNCTION, // function
  IF,       // if
  // IN
  // INSTANCEOF
  NEW,    // new
  RETURN, // return
  SWITCH, // switch
  THROW,  // throw
  TRY,    // try
  // TYPEOF
  VAR, // var
  // VOID
  WHILE, // while
  WITH,  // with
  THIS,  // this

  NULL,  // null
  TRUE,  // true
  FALSE, // false
  NUMBER,
  STRING,
  BIGINT,

  // BEGIN Callable
  SUPER, // super
  // BEGIN AnyIdentifier
  IDENTIFIER,
  AWAIT, // await
  YIELD, // yield
  // END AnyIdentifier
  // END Callable
  CLASS,   // class
  CONST,   // const
  EXPORT,  // export
  EXTENDS, // extends
  IMPORT,  // import
  PRIVATE_IDENTIFIER,

  ENUM, // enum

  ESCAPED_KEYWORD(String),
}

impl TokenType {
  pub fn from_single(c: char) -> Self {
    match c {
      '(' => TokenType::LPAREN,
      ')' => TokenType::RPAREN,
      '{' => TokenType::RBRACE,
      '}' => TokenType::LBRACE,
      '[' => TokenType::RBRACK,
      ']' => TokenType::LBRACK,
      ':' => TokenType::COLON,
      ';' => TokenType::SEMICOLON,
      ',' => TokenType::COMMA,
      '~' => TokenType::BIT_NOT,
      '`' => TokenType::TEMPLATE,
      _ => unreachable!("unexpected char"),
    }
  }
}

pub struct TokenValue {}

pub struct Token {
  pub r#type: TokenType,
  pub value: TokenValue,
  pub start_index: usize,
  pub end_index: usize,
  pub line: usize,
  pub column: usize,
  pub had_line_terminator_before: bool,
  pub had_escaped: bool,
}

impl Token {
  pub fn is_automatic_semicolon(&self) -> bool {
    match self.r#type {
      TokenType::SEMICOLON | TokenType::RBRACE | TokenType::EOS => true,
      _ => false,
    }
  }

  pub fn is_member(&self) -> bool {
    match self.r#type {
      TokenType::TEMPLATE | TokenType::PERIOD | TokenType::LBRACK => true,
      _ => false,
    }
  }

  pub fn is_property_call(&self) -> bool {
    match self.r#type {
      TokenType::TEMPLATE
      | TokenType::PERIOD
      | TokenType::LBRACK
      | TokenType::OPTIONAL
      | TokenType::LPAREN => true,
      _ => false,
    }
  }

  pub fn is_keyword(&self) -> bool {
    match self.r#type {
      TokenType::AWAIT
      | TokenType::BREAK
      | TokenType::CASE
      | TokenType::CATCH
      | TokenType::CLASS
      | TokenType::CONST
      | TokenType::CONTINUE
      | TokenType::DEBUGGER
      | TokenType::DEFAULT
      | TokenType::DELETE
      | TokenType::DO
      | TokenType::ELSE
      | TokenType::ENUM
      | TokenType::EXPORT
      | TokenType::EXTENDS
      | TokenType::FALSE
      | TokenType::FINALLY
      | TokenType::FOR
      | TokenType::FUNCTION
      | TokenType::IF
      | TokenType::IMPORT
      | TokenType::IN
      | TokenType::INSTANCEOF
      | TokenType::NEW
      | TokenType::NULL
      | TokenType::RETURN
      | TokenType::SUPER
      | TokenType::SWITCH
      | TokenType::THIS
      | TokenType::THROW
      | TokenType::TRUE
      | TokenType::TRY
      | TokenType::TYPEOF
      | TokenType::VAR
      | TokenType::VOID
      | TokenType::WHILE
      | TokenType::WITH
      | TokenType::YIELD => true,
      _ => false,
    }
  }
}

pub fn is_reserved_word_strict(s: &str) -> bool {
  match s {
    "implements" | "interface" | "let" | "package" | "private"
    | "protected" | "public" | "static" | "yield" => true,
    _ => false,
  }
}

pub fn lookup_keyword(s: &str, has_escaped: bool) -> Option<TokenType> {
  match s {
    "await" => Some(TokenType::AWAIT),
    "break" => Some(TokenType::BREAK),
    "case" => Some(TokenType::CASE),
    "catch" => Some(TokenType::CATCH),
    "class" => Some(TokenType::CLASS),
    "const" => Some(TokenType::CONST),
    "continue" => Some(TokenType::CONTINUE),
    "debugger" => Some(TokenType::DEBUGGER),
    "default" => Some(TokenType::DEFAULT),
    "delete" => Some(TokenType::DELETE),
    "do" => Some(TokenType::DO),
    "else" => Some(TokenType::ELSE),
    "enum" => Some(TokenType::ENUM),
    "export" => Some(TokenType::EXPORT),
    "extends" => Some(TokenType::EXTENDS),
    "false" => Some(TokenType::FALSE),
    "finally" => Some(TokenType::FINALLY),
    "for" => Some(TokenType::FOR),
    "function" => Some(TokenType::FUNCTION),
    "if" => Some(TokenType::IF),
    "import" => Some(TokenType::IMPORT),
    "in" => Some(TokenType::IN),
    "instanceof" => Some(TokenType::INSTANCEOF),
    "new" => Some(TokenType::NEW),
    "null" => Some(TokenType::NULL),
    "return" => Some(TokenType::RETURN),
    "super" => Some(TokenType::SUPER),
    "switch" => Some(TokenType::SWITCH),
    "this" => Some(TokenType::THIS),
    "throw" => Some(TokenType::THROW),
    "true" => Some(TokenType::TRUE),
    "try" => Some(TokenType::TRY),
    "typeof" => Some(TokenType::TYPEOF),
    "var" => Some(TokenType::VAR),
    "void" => Some(TokenType::VOID),
    "while" => Some(TokenType::WHILE),
    "with" => Some(TokenType::WITH),
    "yield" => Some(TokenType::YIELD),
    _ if has_escaped => Some(TokenType::ESCAPED_KEYWORD(s.to_owned())),
    _ => None,
  }
}
