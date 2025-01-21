use crate::converter::{Converter, IndexWithConverter};

use crate::character::Character;
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

    fn len(&self) -> u64;

    fn get_l_backward(&self, i: u64) -> Self::T;
    fn lf_map_backward(&self, i: u64) -> u64;
    fn lf_map2_backward(&self, c: Self::T, i: u64) -> u64;

    fn iter_backward(&self, i: u64) -> BackwardIterator<Self> {
        debug_assert!(i < self.len());
        BackwardIterator { index: self, i }
    }

    fn get_f_forward(&self, i: u64) -> Self::T;
    fn fl_map_forward(&self, i: u64) -> u64;
    fn fl_map2_forward(&self, c: Self::T, i: u64) -> u64;

    #[doc(hidden)]
    fn iter_forward(&self, i: u64) -> ForwardIterator<Self> {
        debug_assert!(i < self.len());
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
        let c = self.index.get_l_backward(self.i);
        self.i = self.index.lf_map_backward(self.i);
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
        let c = self.index.get_f_forward(self.i);
        self.i = self.index.fl_map_forward(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}
