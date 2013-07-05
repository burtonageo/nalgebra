use std::uint::iterate;
use std::num::{One, Zero};
use std::vec::from_elem;
use std::cmp::ApproxEq;
use std::iterator::IteratorUtil;
use traits::inv::Inv;
use traits::division_ring::DivisionRing;
use traits::transpose::Transpose;
use traits::rlmul::{RMul, LMul};
use dvec::{DVec, zero_vec_with_dim};

#[deriving(Eq, ToStr, Clone)]
pub struct DMat<N>
{
  dim: uint, // FIXME: handle more than just square matrices
  mij: ~[N]
}

#[inline]
pub fn zero_mat_with_dim<N: Zero + Copy>(dim: uint) -> DMat<N>
{ DMat { dim: dim, mij: from_elem(dim * dim, Zero::zero()) } }

#[inline]
pub fn is_zero_mat<N: Zero>(mat: &DMat<N>) -> bool
{ mat.mij.iter().all(|e| e.is_zero()) }

#[inline]
pub fn one_mat_with_dim<N: Copy + One + Zero>(dim: uint) -> DMat<N>
{
  let mut res = zero_mat_with_dim(dim);
  let     _1  = One::one::<N>();

  for iterate(0u, dim) |i|
  { res.set(i, i, &_1); }

  res
}

impl<N: Copy> DMat<N>
{
  #[inline]
  pub fn offset(&self, i: uint, j: uint) -> uint
  { i * self.dim + j }

  #[inline]
  pub fn set(&mut self, i: uint, j: uint, t: &N)
  {
    assert!(i < self.dim);
    assert!(j < self.dim);
    self.mij[self.offset(i, j)] = copy *t
  }

  #[inline]
  pub fn at(&self, i: uint, j: uint) -> N
  {
    assert!(i < self.dim);
    assert!(j < self.dim);
    copy self.mij[self.offset(i, j)]
  }
}

impl<N: Copy> Index<(uint, uint), N> for DMat<N>
{
  #[inline]
  fn index(&self, &(i, j): &(uint, uint)) -> N
  { self.at(i, j) }
}

impl<N: Copy + Mul<N, N> + Add<N, N> + Zero>
Mul<DMat<N>, DMat<N>> for DMat<N>
{
  fn mul(&self, other: &DMat<N>) -> DMat<N>
  {
    assert!(self.dim == other.dim);

    let     dim = self.dim;
    let mut res = zero_mat_with_dim(dim);

    for iterate(0u, dim) |i|
    {
      for iterate(0u, dim) |j|
      {
        let mut acc = Zero::zero::<N>();

        for iterate(0u, dim) |k|
        { acc = acc + self.at(i, k) * other.at(k, j); }

        res.set(i, j, &acc);
      }
    }

    res
  }
}

impl<N: Copy + Add<N, N> + Mul<N, N> + Zero>
RMul<DVec<N>> for DMat<N>
{
  fn rmul(&self, other: &DVec<N>) -> DVec<N>
  {
    assert!(self.dim == other.at.len());

    let     dim           = self.dim;
    let mut res : DVec<N> = zero_vec_with_dim(dim);

    for iterate(0u, dim) |i|
    {
      for iterate(0u, dim) |j|
      { res.at[i] = res.at[i] + other.at[j] * self.at(i, j); }
    }

    res
  }
}

impl<N: Copy + Add<N, N> + Mul<N, N> + Zero>
LMul<DVec<N>> for DMat<N>
{
  fn lmul(&self, other: &DVec<N>) -> DVec<N>
  {
    assert!(self.dim == other.at.len());

    let     dim           = self.dim;
    let mut res : DVec<N> = zero_vec_with_dim(dim);

    for iterate(0u, dim) |i|
    {
      for iterate(0u, dim) |j|
      { res.at[i] = res.at[i] + other.at[j] * self.at(j, i); }
    }

    res
  }
}

impl<N: Copy + Eq + DivisionRing>
Inv for DMat<N>
{
  #[inline]
  fn inverse(&self) -> Option<DMat<N>>
  {
    let mut res : DMat<N> = copy *self;

    if res.invert()
    { Some(res) }
    else
    { None }
  }

  fn invert(&mut self) -> bool
  {
    let     dim = self.dim;
    let mut res = one_mat_with_dim::<N>(dim);
    let     _0T = Zero::zero::<N>();

    // inversion using Gauss-Jordan elimination
    for iterate(0u, dim) |k|
    {
      // search a non-zero value on the k-th column
      // FIXME: would it be worth it to spend some more time searching for the
      // max instead?

      let mut n0 = k; // index of a non-zero entry

      while (n0 != dim)
      {
        if self.at(n0, k) != _0T
        { break; }

        n0 = n0 + 1;
      }

      if n0 == dim
      { return false }

      // swap pivot line
      if n0 != k
      {
        for iterate(0u, dim) |j|
        {
          let off_n0_j = self.offset(n0, j);
          let off_k_j  = self.offset(k, j);

          self.mij.swap(off_n0_j, off_k_j);
          res.mij.swap(off_n0_j, off_k_j);
        }
      }

      let pivot = self.at(k, k);

      for iterate(k, dim) |j|
      {
        let selfval = &(self.at(k, j) / pivot);
        self.set(k, j, selfval);
      }

      for iterate(0u, dim) |j|
      {
        let resval  = &(res.at(k, j)   / pivot);
        res.set(k, j, resval);
      }

      for iterate(0u, dim) |l|
      {
        if l != k
        {
          let normalizer = self.at(l, k);

          for iterate(k, dim) |j|
          {
            let selfval = &(self.at(l, j) - self.at(k, j) * normalizer);
            self.set(l, j, selfval);
          }

          for iterate(0u, dim) |j|
          {
            let resval  = &(res.at(l, j)   - res.at(k, j)   * normalizer);
            res.set(l, j, resval);
          }
        }
      }
    }

    *self = res;

    true
  }
}

impl<N:Copy> Transpose for DMat<N>
{
  #[inline]
  fn transposed(&self) -> DMat<N>
  {
    let mut res = copy *self;

    res.transpose();

    res
  }

  fn transpose(&mut self)
  {
    let dim = self.dim;

    for iterate(1u, dim) |i|
    {
      for iterate(0u, dim - 1) |j|
      {
        let off_i_j = self.offset(i, j);
        let off_j_i = self.offset(j, i);

        self.mij.swap(off_i_j, off_j_i);
      }
    }
  }
}

impl<N: ApproxEq<N>> ApproxEq<N> for DMat<N>
{
  #[inline]
  fn approx_epsilon() -> N
  { ApproxEq::approx_epsilon::<N, N>() }

  #[inline]
  fn approx_eq(&self, other: &DMat<N>) -> bool
  {
    let mut zip = self.mij.iter().zip(other.mij.iter());

    do zip.all |(a, b)| { a.approx_eq(b) }
  }

  #[inline]
  fn approx_eq_eps(&self, other: &DMat<N>, epsilon: &N) -> bool
  {
    let mut zip = self.mij.iter().zip(other.mij.iter());

    do zip.all |(a, b)| { a.approx_eq_eps(b, epsilon) }
  }
}