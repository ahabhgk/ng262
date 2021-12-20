pub mod abstract_operations;
pub mod agent;
pub mod helpers;
pub mod language_types;
pub mod parser;
pub mod runtime_semantics;
pub mod specification_types;
pub mod static_semantics;

#[cfg(test)]
mod tests {
  use std::path::Path;

  use super::*;

  #[test]
  fn it_works() {
    let file = parser::parse(Path::new("./index.js"), false);
    dbg!(file.unwrap());
  }
}
