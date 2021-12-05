use std::str::Chars;

pub struct Source<'a> {
  iter: Chars<'a>,
  index: usize,
}

impl<'a> Source<'a> {
  pub fn new(s: &'a str) -> Self {
    Self {
      iter: s.chars(),
      index: 0, // TODO: read_index starts with -1?
    }
  }

  pub fn position(&self) -> usize {
    self.index
  }

  pub fn current(&self) -> Option<char> {
    self.get(self.index)
  }

  pub fn peek(&self) -> Option<char> {
    self.get(self.index + 1)
  }

  pub fn forward(&mut self) {
    self.index += 1;
  }

  pub fn backward(&mut self) {
    self.index -= 1;
  }

  pub fn next(&mut self) -> Option<char> {
    self.forward();
    self.current()
  }

  pub fn get(&self, i: usize) -> Option<char> {
    self.iter.clone().nth(i)
  }

  pub fn index_of(&self, c: char) -> Option<usize> {
    for (i, ch) in self.iter.clone().skip(self.index).enumerate() {
      if ch == c {
        return Some(i + self.index);
      }
    }
    None
  }

  pub fn slice(&self, start: usize, end: usize) -> String {
    let str = self.iter.as_str();
    let str = &str[start..end];
    str.to_owned()
  }
}
