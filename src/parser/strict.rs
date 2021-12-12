pub trait IsStrict {
  fn is_strict(&self) -> bool;
}

pub trait SetStrict: IsStrict {
  fn set_strict(&mut self, is_strict: bool);

  fn strict_on(&mut self) {
    self.set_strict(true);
  }

  fn strict_off(&mut self) {
    self.set_strict(false);
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

  pub fn set_strict(&mut self, is_strict: bool) {
    self.0 = is_strict;
  }
}
