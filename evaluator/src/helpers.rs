#[derive(Debug, Clone, Copy)]
pub enum Either<A, B> {
  A(A),
  B(B),
}
