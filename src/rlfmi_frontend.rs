use serde::{Deserialize, Serialize};

use crate::frontend::{HasPosition, Search, SearchIndex, SearchWithLocate};
use crate::rlfmi::RLFMIndex as RLFMIndexBackend;
use crate::search::Search as SearchBackend;
use crate::suffix_array::{self, SuffixOrderSampledArray};
use crate::SearchIndexWithLocate;
use crate::{character::Character, converter::Converter};

/// A Run-Length FM-index.
///
/// This can be more space-efficient than the FM-index, but is slower.
#[derive(Serialize, Deserialize)]
pub struct RLFMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    backend: RLFMIndexBackend<T, C, S>,
}

impl<T, C> RLFMIndex<T, C, ()>
where
    T: Character,
    C: Converter<T>,
{
    /// Create a new RLFM-Index from a text. The index only supports the count
    /// operation.
    ///
    /// - `text` is a vector of [`Character`]s.
    ///
    /// - `converter` is a [`Converter`] is used to convert the characters to a
    ///   smaller alphabet. Use [`converter::IdConverter`] if you don't need to
    ///   restrict the alphabet. Use [`converter::RangeConverter`] if you can
    ///   contrain characters to a particular range. See [`converter`] for more
    ///   details.
    pub fn count_only(text: Vec<T>, converter: C) -> Self {
        Self {
            backend: RLFMIndexBackend::create(text, converter, |_| ()),
        }
    }
}

impl<T, C> RLFMIndex<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    /// Create a new RLFM-Index from a text. The index supports both the count
    /// and locate operations.
    ///
    /// - `text` is a vector of [`Character`]s.
    ///
    /// - `converter` is a [`Converter`] is used to convert the characters to a
    ///   smaller alphabet. Use [`converter::IdConverter`] if you don't need to
    ///   restrict the alphabet. Use [`converter::RangeConverter`] if you can
    ///   contrain characters to a particular range. See [`converter`] for more
    ///   details.
    ///
    /// - `level` is the sampling level to use for position lookup. A sampling
    ///   level of 0 means the most memory is used (a full suffix-array is
    ///   retained), while looking up positions is faster. A sampling level of
    ///   1 means half the memory is used, but looking up positions is slower.
    ///   Each increase in level halves the memory usage but slows down
    ///   position lookup.
    pub fn new(text: Vec<T>, converter: C, level: usize) -> Self {
        Self {
            backend: RLFMIndexBackend::create(text, converter, |sa| {
                suffix_array::sample(sa, level)
            }),
        }
    }
}

impl<T, C> HasPosition for RLFMIndex<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
}

impl<T, C, S> RLFMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    pub fn search<K>(&self, pattern: K) -> RLFMIndexSearch<T, C, S>
    where
        K: AsRef<[T]>,
    {
        RLFMIndexSearch::new(self.backend.search(pattern))
    }

    /// The length of the text.
    pub fn len(&self) -> u64 {
        self.backend.len()
    }
}

impl<T, C, S> SearchIndex<T> for RLFMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    #[allow(refining_impl_trait)]
    fn search<K>(&self, pattern: K) -> RLFMIndexSearch<T, C, S>
    where
        K: AsRef<[T]>,
    {
        RLFMIndex::search(self, pattern)
    }
}

impl<T, C> SearchIndexWithLocate<T> for RLFMIndex<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    #[allow(refining_impl_trait)]
    fn search<K>(&self, pattern: K) -> RLFMIndexSearch<T, C, SuffixOrderSampledArray>
    where
        K: AsRef<[T]>,
    {
        RLFMIndex::search(self, pattern)
    }
}

pub struct RLFMIndexSearch<'a, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    search_backend: SearchBackend<'a, RLFMIndexBackend<T, C, S>>,
}

impl<'a, T, C, S> RLFMIndexSearch<'a, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    fn new(search_backend: SearchBackend<'a, RLFMIndexBackend<T, C, S>>) -> Self {
        RLFMIndexSearch { search_backend }
    }

    /// Search in the current search result, refining it.
    ///
    /// This adds a prefix `pattern` to the existing pattern, and
    /// looks for those expanded patterns in the text.
    pub fn search<K>(&self, pattern: K) -> Self
    where
        K: AsRef<[T]>,
    {
        let search_backend = self.search_backend.search(pattern);
        RLFMIndexSearch { search_backend }
    }

    /// Get the number of matches.
    pub fn count(&self) -> u64 {
        self.search_backend.count()
    }

    /// Get an iterator that goes backwards through the text, producing
    /// [`Character`].
    pub fn iter_backward(&self, i: u64) -> impl Iterator<Item = T> + '_ {
        self.search_backend.iter_backward(i)
    }

    /// Get an iterator that goes forwards through the text, producing
    /// [`Character`].
    pub fn iter_forward(&self, i: u64) -> impl Iterator<Item = T> + '_ {
        self.search_backend.iter_forward(i)
    }
}

impl<T, C, S> Search<T> for RLFMIndexSearch<'_, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    fn search<K>(&self, pattern: K) -> Self
    where
        K: AsRef<[T]>,
    {
        RLFMIndexSearch::search(self, pattern)
    }

    fn count(&self) -> u64 {
        RLFMIndexSearch::count(self)
    }

    fn iter_backward(&self, i: u64) -> impl Iterator<Item = T> + '_ {
        RLFMIndexSearch::iter_backward(self, i)
    }

    fn iter_forward(&self, i: u64) -> impl Iterator<Item = T> + '_ {
        RLFMIndexSearch::iter_forward(self, i)
    }
}

impl<T, C> RLFMIndexSearch<'_, T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    /// List the position of all occurrences.
    pub fn locate(&self) -> Vec<u64> {
        self.search_backend.locate()
    }
}

impl<T, C> SearchWithLocate<T> for RLFMIndexSearch<'_, T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    fn locate(&self) -> Vec<u64> {
        RLFMIndexSearch::locate(self)
    }
}
