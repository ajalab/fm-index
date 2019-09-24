use crate::converter::{Converter, IndexWithConverter};
use crate::suffix_array::IndexWithSA;

pub trait BackwardIterableIndex: Sized {
    type T: Copy + Clone;
    fn get_l(&self, i: u64) -> Self::T;
    fn lf_map(&self, i: u64) -> u64 {
        self.lf_map2(self.get_l(i), i)
    }
    fn lf_map2(&self, c: Self::T, i: u64) -> u64;
    fn len(&self) -> u64;

    fn search_backward<'a, K>(&'a self, pattern: K) -> Search<'a, Self>
    where
        K: AsRef<[Self::T]>,
    {
        Search::new(self).search_backward(pattern)
    }

    fn iter_backward<'a>(&'a self, i: u64) -> BackwardIterator<'a, Self> {
        debug_assert!(i < self.len());
        BackwardIterator { index: self, i }
    }
}

pub struct BackwardIterator<'a, I>
where
    I: BackwardIterableIndex,
{
    index: &'a I,
    i: u64,
}

impl<'a, T, I> Iterator for BackwardIterator<'a, I>
where
    T: Copy + Clone,
    I: BackwardIterableIndex<T = T> + IndexWithConverter<T>,
{
    type Item = <I as BackwardIterableIndex>::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_l(self.i);
        self.i = self.index.lf_map2(c, self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}

pub struct Search<'a, I>
where
    I: BackwardIterableIndex,
{
    index: &'a I,
    s: u64,
    e: u64,
    pattern: Vec<I::T>,
}

impl<'a, I> Search<'a, I>
where
    I: BackwardIterableIndex,
{
    fn new(index: &'a I) -> Search<I> {
        Search {
            index: index,
            s: 0,
            e: index.len(),
            pattern: vec![],
        }
    }

    pub fn search_backward<K: AsRef<[I::T]>>(&self, pattern: K) -> Self {
        let mut s = self.s;
        let mut e = self.e;
        let mut pattern = pattern.as_ref().to_vec();
        for &c in pattern.iter().rev() {
            s = self.index.lf_map2(c, s);
            e = self.index.lf_map2(c, e);
            if s == e {
                break;
            }
        }
        pattern.extend_from_slice(&self.pattern);

        Search {
            index: self.index,
            s,
            e,
            pattern,
        }
    }

    pub fn get_range(&self) -> (u64, u64) {
        (self.s, self.e)
    }

    pub fn count(&self) -> u64 {
        self.e - self.s
    }
}

impl<'a, I> Search<'a, I>
where
    I: BackwardIterableIndex + IndexWithSA,
{
    pub fn locate(&self) -> Vec<u64> {
        let mut results: Vec<u64> = Vec::with_capacity((self.e - self.s) as usize);
        for k in self.s..self.e {
            results.push(self.index.get_sa(k));
        }
        results
    }
}

pub trait ForwardIterableIndex: Sized {
    type T: Copy + Clone;
    fn get_f(&self, i: u64) -> Self::T;
    fn inverse_lf_map(&self, i: u64) -> u64 {
        self.inverse_lf_map2(self.get_f(i), i)
    }
    fn inverse_lf_map2(&self, c: Self::T, i: u64) -> u64;
    fn len(&self) -> u64;

    fn iter_forward<'a>(&'a self, i: u64) -> ForwardIterator<'a, Self> {
        debug_assert!(i < self.len());
        ForwardIterator { index: self, i }
    }
}

pub struct ForwardIterator<'a, I>
where
    I: ForwardIterableIndex,
{
    index: &'a I,
    i: u64,
}

impl<'a, T, I> Iterator for ForwardIterator<'a, I>
where
    T: Copy + Clone,
    I: ForwardIterableIndex<T = T> + IndexWithConverter<T>,
{
    type Item = <I as ForwardIterableIndex>::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_f(self.i);
        self.i = self.index.inverse_lf_map2(c, self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}
