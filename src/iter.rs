use crate::character::Character;
use crate::converter::{Converter, IndexWithConverter};
use crate::seal;
use crate::search::Search;

/// A FM-Index that can search texts backwards and forwards.
pub trait FMIndex: Sized + seal::Sealed {
    /// A [`Character`] type.
    type T: Character;

    #[doc(hidden)]
    fn len<L: seal::IsLocal>(&self) -> u64;

    #[doc(hidden)]
    fn get_l<L: seal::IsLocal>(&self, i: u64) -> Self::T;
    #[doc(hidden)]
    fn lf_map<L: seal::IsLocal>(&self, i: u64) -> u64;
    #[doc(hidden)]
    fn lf_map2<L: seal::IsLocal>(&self, c: Self::T, i: u64) -> u64;
    #[doc(hidden)]
    fn get_f<L: seal::IsLocal>(&self, i: u64) -> Self::T;
    #[doc(hidden)]
    fn fl_map<L: seal::IsLocal>(&self, i: u64) -> u64;
    #[doc(hidden)]
    fn fl_map2<L: seal::IsLocal>(&self, c: Self::T, i: u64) -> u64;

    #[doc(hidden)]
    fn search<K>(&self, pattern: K) -> Search<Self>
    where
        K: AsRef<[Self::T]>,
    {
        Search::new(self).search(pattern)
    }

    #[doc(hidden)]
    fn iter_forward<L: seal::IsLocal>(&self, i: u64) -> ForwardIterator<Self> {
        debug_assert!(i < self.len::<L>());
        ForwardIterator { index: self, i }
    }

    #[doc(hidden)]
    fn iter_backward<L: seal::IsLocal>(&self, i: u64) -> BackwardIterator<Self> {
        debug_assert!(i < self.len::<seal::Local>());
        BackwardIterator { index: self, i }
    }
}

/// An iterator that goes backwards through the text, producing [`Character`].
pub struct BackwardIterator<'a, I>
where
    I: FMIndex,
{
    index: &'a I,
    i: u64,
}

impl<T, I> Iterator for BackwardIterator<'_, I>
where
    T: Character,
    I: FMIndex<T = T> + IndexWithConverter<T>,
{
    type Item = <I as FMIndex>::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_l::<seal::Local>(self.i);
        self.i = self.index.lf_map::<seal::Local>(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}

/// An iterator that goes forwards through the text, producing [`Character`].
pub struct ForwardIterator<'a, I>
where
    I: FMIndex,
{
    index: &'a I,
    i: u64,
}

impl<T, I> Iterator for ForwardIterator<'_, I>
where
    T: Character,
    I: FMIndex<T = T> + IndexWithConverter<T>,
{
    type Item = <I as FMIndex>::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_f::<seal::Local>(self.i);
        self.i = self.index.fl_map::<seal::Local>(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}
