use num_bigint::BigInt;

use super::source::SourceText;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
  // BEGIN PropertyOrCall
  // BEGIN Member
  // BEGIN Template
  /// `
  Template,
  // END Template

  // BEGIN Property
  /// .
  Period,
  /// [
  LBrack,
  // END Property
  // END Member
  /// ?.
  Optional,
  /// (
  LParen,
  // END PropertyOrCall
  /// )
  RParen,
  /// ]
  RBrack,
  /// {
  LBrace,
  /// :
  Colon,
  /// ...
  Ellipsis,
  /// ?
  Conditional,
  // BEGIN AutoSemicolon
  /// ;
  Semicolon,
  /// }
  RBrace,

  EndOfSource,
  // END AutoSemicolon

  // BEGIN ArrowOrAssign
  /// =>
  Arrow,
  // BEGIN Assign
  /// =
  Assign,
  // Logical
  /// ??=
  AssignNullish,
  /// ||=
  AssignOr,
  /// &&=
  AssignAnd,

  // Binop
  /// |=
  AssignBitOr,
  /// ^=
  AssignBitXor,
  /// &=
  AssignBitAnd,
  /// <<=
  AssignShl,
  /// >>=
  AssignSar,
  /// >>>=
  AssignShr,
  /// *=
  AssignMul,
  /// /=
  AssignDiv,
  /// %=
  AssignMod,
  /// **=
  AssignExp,

  // Unop
  /// +=
  AssignAdd,
  /// -=
  AssignSub,
  // END Assign
  // END ArrowOrAssign

  // Binary operators by precidence
  /// ,
  Comma,

  // Logical
  /// ??
  Nullish,
  /// ||
  Or,
  /// &&
  And,

  // Binop
  /// |
  BitOr,
  /// ^
  BitXor,
  /// &
  BitAnd,
  /// <<
  Shl,
  /// >>
  Sar,
  /// >>>
  Shr,
  /// *
  Mul,
  /// /
  Div,
  /// %
  Mod,
  /// **
  Exp,

  // Unop
  /// +
  Add,
  /// -
  Sub,

  /// !
  Not,
  /// ~
  BitNot,
  /// delete
  Delete,
  /// typeof
  Typeof,
  /// void
  Void,

  // BEGIN IsCountOp
  /// ++
  Inc,
  /// --
  Dec,
  // END IsCountOp
  // END IsUnaryOrCountOp
  /// ==
  Equal,
  /// ===
  StrictEqual,
  /// !=
  NotEqual,
  /// !==
  StrictNotEqual,
  /// <
  LessThan,
  /// >
  GreaterThan,
  /// <=
  LessThanEqual,
  /// >=
  GreaterThanEqual,
  /// instanceof
  Instanceof,
  /// in
  In,

  /// break
  Break,
  /// case
  Case,
  /// catch
  Catch,
  /// continue
  Continue,
  /// debugger
  Debugger,
  /// default
  Default,
  /// do
  Do,
  /// else
  Else,
  /// finally
  Finally,
  /// for
  For,
  /// function
  Function,
  /// if
  If,
  /// new
  New,
  /// return
  Return,
  /// switch
  Switch,
  /// throw
  Throw,
  /// try
  Try,
  /// var
  Var,
  /// while
  While,
  /// with
  With,
  /// this
  This,

  /// null
  Null,
  /// true
  True,
  /// false
  False,
  /// number
  Number(f64),
  /// string
  String(String),
  /// bigint
  BigInt(BigInt),

  // BEGIN Callable
  /// super
  Super,
  // BEGIN AnyIdentifier
  /// identifier
  Identifier(String),
  /// await
  Await,
  /// yield
  Yield,
  // END AnyIdentifier
  // END Callable
  /// class
  Class,
  /// const
  Const,
  /// export
  Export,
  /// extends
  Extends,
  /// import
  Import,
  /// private_identifier
  PrivateIdentifier(String),

  /// enum
  Enum,

  EscapedKeyword(String),
}

impl TokenType {
  pub fn from_single(c: char) -> Self {
    match c {
      '(' => TokenType::LParen,
      ')' => TokenType::RParen,
      '{' => TokenType::LBrace,
      '}' => TokenType::RBrace,
      '[' => TokenType::LBrack,
      ']' => TokenType::RBrack,
      ':' => TokenType::Colon,
      ';' => TokenType::Semicolon,
      ',' => TokenType::Comma,
      '~' => TokenType::BitNot,
      '`' => TokenType::Template,
      _ => unreachable!("unexpected char"),
    }
  }

  pub fn is_automatic_semicolon(&self) -> bool {
    matches!(
      self,
      TokenType::Semicolon | TokenType::RBrace | TokenType::EndOfSource
    )
  }

  pub fn is_member(&self) -> bool {
    matches!(
      self,
      TokenType::Template | TokenType::Period | TokenType::LBrack
    )
  }

  pub fn is_property_call(&self) -> bool {
    matches!(
      self,
      TokenType::Template
        | TokenType::Period
        | TokenType::LBrack
        | TokenType::Optional
        | TokenType::LParen
    )
  }

  pub fn is_keyword(&self) -> bool {
    matches!(
      self,
      TokenType::Await
        | TokenType::Break
        | TokenType::Case
        | TokenType::Catch
        | TokenType::Class
        | TokenType::Const
        | TokenType::Continue
        | TokenType::Debugger
        | TokenType::Default
        | TokenType::Delete
        | TokenType::Do
        | TokenType::Else
        | TokenType::Enum
        | TokenType::Export
        | TokenType::Extends
        | TokenType::False
        | TokenType::Finally
        | TokenType::For
        | TokenType::Function
        | TokenType::If
        | TokenType::Import
        | TokenType::In
        | TokenType::Instanceof
        | TokenType::New
        | TokenType::Null
        | TokenType::Return
        | TokenType::Super
        | TokenType::Switch
        | TokenType::This
        | TokenType::Throw
        | TokenType::True
        | TokenType::Try
        | TokenType::Typeof
        | TokenType::Var
        | TokenType::Void
        | TokenType::While
        | TokenType::With
        | TokenType::Yield
    )
  }

  pub fn identifier_or_keyword_value(&self) -> String {
    let s = match self {
      TokenType::Await => "await",
      TokenType::Break => "break",
      TokenType::Case => "case",
      TokenType::Catch => "catch",
      TokenType::Class => "class",
      TokenType::Const => "const",
      TokenType::Continue => "continue",
      TokenType::Debugger => "debugger",
      TokenType::Default => "default",
      TokenType::Delete => "delete",
      TokenType::Do => "do",
      TokenType::Else => "else",
      TokenType::Enum => "enum",
      TokenType::Export => "export",
      TokenType::Extends => "extends",
      TokenType::False => "false",
      TokenType::Finally => "finally",
      TokenType::For => "for",
      TokenType::Function => "function",
      TokenType::If => "if",
      TokenType::Import => "import",
      TokenType::In => "in",
      TokenType::Instanceof => "instanceof",
      TokenType::New => "new",
      TokenType::Null => "null",
      TokenType::Return => "return",
      TokenType::Super => "super",
      TokenType::Switch => "switch",
      TokenType::This => "this",
      TokenType::Throw => "throw",
      TokenType::True => "true",
      TokenType::Try => "try",
      TokenType::Typeof => "typeof",
      TokenType::Var => "var",
      TokenType::Void => "void",
      TokenType::While => "while",
      TokenType::With => "with",
      TokenType::Yield => "yield",
      TokenType::Identifier(s) | TokenType::EscapedKeyword(s) => s,
      _ => panic!("unexpected token_type"),
    };
    s.to_owned()
  }
}

#[derive(Debug, Clone)]
pub struct Token {
  pub token_type: TokenType,
  pub start_index: usize,
  pub end_index: usize,
  pub line: usize,
  pub column: usize,
  pub had_line_terminator_before: bool,
  pub had_escaped: bool,
  pub source_text: String,
}

impl SourceText for Token {
  fn source_text(&self) -> &str {
    self.source_text.as_str()
  }
}

pub fn is_reserved_word_strict(s: &str) -> bool {
  matches!(
    s,
    "implements"
      | "interface"
      | "let"
      | "package"
      | "private"
      | "protected"
      | "public"
      | "static"
      | "yield"
  )
}

fn lookup_unescaped_keyword(s: &str) -> Option<TokenType> {
  match s {
    "await" => Some(TokenType::Await),
    "break" => Some(TokenType::Break),
    "case" => Some(TokenType::Case),
    "catch" => Some(TokenType::Catch),
    "class" => Some(TokenType::Class),
    "const" => Some(TokenType::Const),
    "continue" => Some(TokenType::Continue),
    "debugger" => Some(TokenType::Debugger),
    "default" => Some(TokenType::Default),
    "delete" => Some(TokenType::Delete),
    "do" => Some(TokenType::Do),
    "else" => Some(TokenType::Else),
    "enum" => Some(TokenType::Enum),
    "export" => Some(TokenType::Export),
    "extends" => Some(TokenType::Extends),
    "false" => Some(TokenType::False),
    "finally" => Some(TokenType::Finally),
    "for" => Some(TokenType::For),
    "function" => Some(TokenType::Function),
    "if" => Some(TokenType::If),
    "import" => Some(TokenType::Import),
    "in" => Some(TokenType::In),
    "instanceof" => Some(TokenType::Instanceof),
    "new" => Some(TokenType::New),
    "null" => Some(TokenType::Null),
    "return" => Some(TokenType::Return),
    "super" => Some(TokenType::Super),
    "switch" => Some(TokenType::Switch),
    "this" => Some(TokenType::This),
    "throw" => Some(TokenType::Throw),
    "true" => Some(TokenType::True),
    "try" => Some(TokenType::Try),
    "typeof" => Some(TokenType::Typeof),
    "var" => Some(TokenType::Var),
    "void" => Some(TokenType::Void),
    "while" => Some(TokenType::While),
    "with" => Some(TokenType::With),
    "yield" => Some(TokenType::Yield),
    _ => None,
  }
}

pub fn lookup_keyword(s: &str, had_escaped: bool) -> Option<TokenType> {
  lookup_unescaped_keyword(s).map(|t| {
    if had_escaped {
      TokenType::EscapedKeyword(s.to_owned())
    } else {
      t
    }
  })
}
