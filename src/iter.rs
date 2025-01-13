use crate::converter::{Converter, IndexWithConverter};

use crate::character::Character;

/// A search index that can be searched backwards.
pub trait BackwardIterableIndex: Sized {
    /// A [`Character`] type.
    type T: Character;

    fn get_l(&self, i: u64) -> Self::T;
    fn lf_map(&self, i: u64) -> u64;
    fn lf_map2(&self, c: Self::T, i: u64) -> u64;
    fn len(&self) -> u64;

    fn iter_backward(&self, i: u64) -> BackwardIterator<Self> {
        debug_assert!(i < self.len());
        BackwardIterator { index: self, i }
    }
}

/// An iterator that goes backwards through the text, producing [`Character`].
pub struct BackwardIterator<'a, I>
where
    I: BackwardIterableIndex,
{
    index: &'a I,
    i: u64,
}

impl<T, I> Iterator for BackwardIterator<'_, I>
where
    T: Character,
    I: BackwardIterableIndex<T = T> + IndexWithConverter<T>,
{
    type Item = <I as BackwardIterableIndex>::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_l(self.i);
        self.i = self.index.lf_map(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}

/// A search index that can be searched forwards.
pub trait ForwardIterableIndex: Sized {
    /// A [`Character`] type.
    type T: Character;

    fn get_f(&self, i: u64) -> Self::T;
    fn fl_map(&self, i: u64) -> u64;
    fn fl_map2(&self, c: Self::T, i: u64) -> u64;
    fn len(&self) -> u64;

    fn iter_forward(&self, i: u64) -> ForwardIterator<Self> {
        debug_assert!(i < self.len());
        ForwardIterator { index: self, i }
    }
}

/// An iterator that goes forwards through the text, producing [`Character`].
pub struct ForwardIterator<'a, I>
where
    I: ForwardIterableIndex,
{
    index: &'a I,
    i: u64,
}

impl<T, I> Iterator for ForwardIterator<'_, I>
where
    T: Character,
    I: ForwardIterableIndex<T = T> + IndexWithConverter<T>,
{
    type Item = <I as ForwardIterableIndex>::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_f(self.i);
        self.i = self.index.fl_map(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}
