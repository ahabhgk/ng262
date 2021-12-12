use std::collections::HashSet;

use super::strict::{IsStrict, SetStrict, Strict};

pub enum Flag {
  Return = 1 << 0,
  Await = 1 << 1,
  Yield = 1 << 2,
  Parameters = 1 << 3,
  NewTarget = 1 << 4,
  ImportMeta = 1 << 5,
  SuperCall = 1 << 6,
  SuperProperty = 1 << 7,
  In = 1 << 8,
  Default = 1 << 9,
  Module = 1 << 10,
}

impl From<Flag> for u16 {
  fn from(f: Flag) -> Self {
    f as u16
  }
}

#[derive(Default)]
pub struct Flags(u16);

impl Flags {
  pub fn add(&mut self, flag: Flag) {
    self.0 |= u16::from(flag)
  }

  pub fn delete(&mut self, flag: Flag) {
    self.0 &= !u16::from(flag)
  }

  pub fn has(&self, flag: Flag) -> bool {
    (self.0 & u16::from(flag)) != 0
  }
}

struct Scope {
  flags: Flags,
  lexicals: HashSet<String>,
  variables: HashSet<String>,
  functions: HashSet<String>,
  parameters: HashSet<String>,
}

impl Scope {
  pub fn new(flags: Flags) -> Self {
    Self {
      flags,
      lexicals: HashSet::new(),
      variables: HashSet::new(),
      functions: HashSet::new(),
      parameters: HashSet::new(),
    }
  }
}

pub struct Resolver {
  scope_stack: Vec<Scope>,
  strict: Strict,
  pub flags: Flags,
}

impl IsStrict for Resolver {
  fn is_strict(&self) -> bool {
    self.strict.is_strict()
  }
}

impl SetStrict for Resolver {
  fn set_strict(&mut self, is_strict: bool) {
    self.strict.set_strict(is_strict);
  }
}
