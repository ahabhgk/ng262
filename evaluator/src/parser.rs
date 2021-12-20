use std::{error::Error, fmt::Display, path::Path, rc::Rc};

use swc_common::{
  errors::{ColorConfig, Handler},
  SourceMap,
};
use swc_ecma_ast::{EsVersion, Program};
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};

#[derive(Debug)]
pub struct ParseError;

impl Error for ParseError {}

impl Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "ParseError")
  }
}

pub fn parse(path: &Path, is_module: bool) -> Result<Program, ParseError> {
  let cm = Rc::new(SourceMap::default());
  let fm = cm.load_file(path).unwrap();
  let handler =
    Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm));
  let lexer = Lexer::new(
    Syntax::Es(EsConfig {
      static_blocks: true,
      ..Default::default()
    }),
    EsVersion::latest(),
    StringInput::from(&*fm),
    None,
  );
  let mut parser = Parser::new_from(lexer);
  let mut error = false;

  let program_result = match is_module {
    true => parser.parse_module().map(Program::Module),
    false => parser.parse_script().map(Program::Script),
  };

  for e in parser.take_errors() {
    e.into_diagnostic(&handler).emit();
    error = true;
  }

  let program = program_result.map_err(|e| {
    e.into_diagnostic(&handler).emit();
    ParseError
  })?;

  // recoverable error
  if error {
    return Err(ParseError);
  }

  Ok(program)
}
