pub struct Strict(bool);

impl Strict {
  pub fn new(b: bool) -> Self {
    Self(b)
  }

  pub fn mode_on(&mut self) {
    self.0 = true;
  }

  pub fn mode_off(&mut self) {
    self.0 = false;
  }

  pub fn is_strict_mode(&self) -> bool {
    self.0
  }
}
