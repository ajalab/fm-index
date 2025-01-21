use crate::converter::{Converter, IndexWithConverter};

use crate::character::Character;
use crate::seal;
use crate::search::Search;

/// A search index backend.
pub(crate) trait SearchIndexBackend: Sized {
    /// A [`Character`] type.
    type T: Character;

    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    fn search<K>(&self, pattern: K) -> Search<Self>
    where
        K: AsRef<[Self::T]>,
    {
        Search::new(self).search(pattern)
    }

    #[doc(hidden)]
    fn len<L: seal::IsLocal>(&self) -> u64;

    #[doc(hidden)]
    fn get_l_backward<L: seal::IsLocal>(&self, i: u64) -> Self::T;
    #[doc(hidden)]
    fn lf_map_backward<L: seal::IsLocal>(&self, i: u64) -> u64;
    #[doc(hidden)]
    fn lf_map2_backward<L: seal::IsLocal>(&self, c: Self::T, i: u64) -> u64;

    #[doc(hidden)]
    fn iter_backward<L: seal::IsLocal>(&self, i: u64) -> BackwardIterator<Self> {
        debug_assert!(i < self.len::<seal::Local>());
        BackwardIterator { index: self, i }
    }

    #[doc(hidden)]
    fn get_f_forward<L: seal::IsLocal>(&self, i: u64) -> Self::T;
    #[doc(hidden)]
    fn fl_map_forward<L: seal::IsLocal>(&self, i: u64) -> u64;
    #[doc(hidden)]
    fn fl_map2_forward<L: seal::IsLocal>(&self, c: Self::T, i: u64) -> u64;

    #[doc(hidden)]
    fn iter_forward<L: seal::IsLocal>(&self, i: u64) -> ForwardIterator<Self> {
        debug_assert!(i < self.len::<L>());
        ForwardIterator { index: self, i }
    }
}

/// An iterator that goes backwards through the text, producing [`Character`].
pub(crate) struct BackwardIterator<'a, I>
where
    I: SearchIndexBackend,
{
    index: &'a I,
    i: u64,
}

impl<T, I> Iterator for BackwardIterator<'_, I>
where
    T: Character,
    I: SearchIndexBackend<T = T> + IndexWithConverter<T>,
{
    type Item = <I as SearchIndexBackend>::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_l_backward::<seal::Local>(self.i);
        self.i = self.index.lf_map_backward::<seal::Local>(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}

/// An iterator that goes forwards through the text, producing [`Character`].
pub(crate) struct ForwardIterator<'a, I>
where
    I: SearchIndexBackend,
{
    index: &'a I,
    i: u64,
}

impl<T, I> Iterator for ForwardIterator<'_, I>
where
    T: Character,
    I: SearchIndexBackend<T = T> + IndexWithConverter<T>,
{
    type Item = <I as SearchIndexBackend>::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_f_forward::<seal::Local>(self.i);
        self.i = self.index.fl_map_forward::<seal::Local>(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}
