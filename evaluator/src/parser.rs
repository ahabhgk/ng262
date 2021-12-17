use std::{path::Path, sync::Arc};

use anyhow::Error;
use swc::try_with_handler;
use swc_common::{FilePathMapping, SourceMap};
use swc_ecma_ast::{EsVersion, Program};
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};
use swc_estree_ast::File;
use swc_estree_compat::babelify::Babelify;
use swc_node_comments::SwcComments;

pub fn parse(path: &Path) -> File {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
  let fm = cm.load_file(path).unwrap();

  let program = try_with_handler(cm.clone(), false, |handler| {
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

    // let program_result = match is_module {
    //   IsModule::Bool(true) => parser.parse_module().map(Program::Module),
    //   IsModule::Bool(false) => parser.parse_script().map(Program::Script),
    //   IsModule::Unknown => parser.parse_program(),
    // };
    let program_result = parser.parse_script().map(Program::Script);

    for e in parser.take_errors() {
      e.into_diagnostic(handler).emit();
      error = true;
    }

    let program = program_result.map_err(|e| {
      e.into_diagnostic(handler).emit();
      Error::msg("Syntax Error")
    })?;

    if error {
      return Err(anyhow::anyhow!("Syntax Error").context(
        "error was recoverable, but proceeding would result in wrong codegen",
      ));
    }

    Ok(program)
  })
  .unwrap();

  let ctx = swc_estree_compat::babelify::Context {
    fm,
    cm,
    comments: SwcComments::default(),
  };
  program.babelify(&ctx)
}
