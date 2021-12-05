pub trait UseStrict {
  fn is_strict(&self) -> bool;

  fn use_strict(&mut self, is_strict: bool);

  fn strict_on(&mut self) {
    self.use_strict(true);
  }

  fn strict_off(&mut self) {
    self.use_strict(false);
  }
}

pub struct Strict(bool);

impl Strict {
  pub fn new(b: bool) -> Self {
    Self(b)
  }

  pub fn is_strict(&self) -> bool {
    self.0
  }

  pub fn use_strict(&mut self, is_strict: bool) {
    self.0 = is_strict;
  }
}
