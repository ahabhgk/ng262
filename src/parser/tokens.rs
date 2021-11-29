#[allow(non_camel_case_types)]
pub enum Token {
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

  ESCAPED_KEYWORD,
}

impl Token {
  pub fn is_automatic_semicolon(&self) -> bool {
    match self {
      Token::SEMICOLON | Token::RBRACE | Token::EOS => true,
      _ => false,
    }
  }

  pub fn is_member(&self) -> bool {
    match self {
      Token::TEMPLATE | Token::PERIOD | Token::LBRACK => true,
      _ => false,
    }
  }

  pub fn is_property_call(&self) -> bool {
    match self {
      Token::TEMPLATE
      | Token::PERIOD
      | Token::LBRACK
      | Token::OPTIONAL
      | Token::LPAREN => true,
      _ => false,
    }
  }

  pub fn is_keyword(&self) -> bool {
    match self {
      Token::AWAIT
      | Token::BREAK
      | Token::CASE
      | Token::CATCH
      | Token::CLASS
      | Token::CONST
      | Token::CONTINUE
      | Token::DEBUGGER
      | Token::DEFAULT
      | Token::DELETE
      | Token::DO
      | Token::ELSE
      | Token::ENUM
      | Token::EXPORT
      | Token::EXTENDS
      | Token::FALSE
      | Token::FINALLY
      | Token::FOR
      | Token::FUNCTION
      | Token::IF
      | Token::IMPORT
      | Token::IN
      | Token::INSTANCEOF
      | Token::NEW
      | Token::NULL
      | Token::RETURN
      | Token::SUPER
      | Token::SWITCH
      | Token::THIS
      | Token::THROW
      | Token::TRUE
      | Token::TRY
      | Token::TYPEOF
      | Token::VAR
      | Token::VOID
      | Token::WHILE
      | Token::WITH
      | Token::YIELD => true,
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
