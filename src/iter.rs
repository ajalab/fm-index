use crate::converter::{Converter, IndexWithConverter};

use crate::character::Character;
use crate::seal;

/// A search index that can be searched backwards.
pub trait BackwardIterableIndex: Sized {
    /// A [`Character`] type.
    type T: Character;

    #[doc(hidden)]
    fn get_l_backward<L: seal::IsLocal>(&self, i: u64) -> Self::T;
    #[doc(hidden)]
    fn lf_map_backward<L: seal::IsLocal>(&self, i: u64) -> u64;
    #[doc(hidden)]
    fn lf_map2_backward<L: seal::IsLocal>(&self, c: Self::T, i: u64) -> u64;
    #[doc(hidden)]
    fn len_backward<L: seal::IsLocal>(&self) -> u64;

    #[doc(hidden)]
    fn iter_backward<L: seal::IsLocal>(&self, i: u64) -> BackwardIterator<Self> {
        debug_assert!(i < self.len_backward::<seal::Local>());
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
        let c = self.index.get_l_backward::<seal::Local>(self.i);
        self.i = self.index.lf_map_backward::<seal::Local>(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}

/// A search index that can be searched forwards.
pub trait ForwardIterableIndex: Sized {
    /// A [`Character`] type.
    type T: Character;

    #[doc(hidden)]
    fn get_f_forward<L: seal::IsLocal>(&self, i: u64) -> Self::T;
    #[doc(hidden)]
    fn fl_map_forward<L: seal::IsLocal>(&self, i: u64) -> u64;
    #[doc(hidden)]
    fn fl_map2_forward<L: seal::IsLocal>(&self, c: Self::T, i: u64) -> u64;
    #[doc(hidden)]
    fn len<L: seal::IsLocal>(&self) -> u64;

    #[doc(hidden)]
    fn iter_forward<L: seal::IsLocal>(&self, i: u64) -> ForwardIterator<Self> {
        debug_assert!(i < self.len::<L>());
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
        let c = self.index.get_f_forward::<seal::Local>(self.i);
        self.i = self.index.fl_map_forward::<seal::Local>(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}
