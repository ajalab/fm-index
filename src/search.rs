use crate::converter::{Converter, IndexWithConverter};
use crate::iter::FMIndex;
use crate::suffix_array::{self, HasPosition, SuffixOrderSampledArray};
use crate::{seal, Character, DefaultFMIndex, RLFMIndex};

/// A builder that builds [`SearchIndex`].
pub struct SearchIndexBuilder<I, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    converter: C,
    // We avoid extracting parts into another `type` definition.
    // Also, we use dyn trait in order not to add another type variable for this closure type.
    #[allow(clippy::type_complexity)]
    get_sample: Box<dyn Fn(&[u64]) -> S>,
    _i: std::marker::PhantomData<I>,
    _t: std::marker::PhantomData<T>,
}

impl<T, C> SearchIndexBuilder<(), T, C, ()>
where
    T: Character,
    C: Converter<T>,
{
    /// Create a new [`SearchIndexBuilder`].
    ///
    /// - `converter` is a [`Converter`] is used to convert the characters to a
    ///   smaller alphabet. Use [`converter::IdConverter`] if you don't need to
    ///   restrict the alphabet. Use [`converter::RangeConverter`] if you can
    ///   contrain characters to a particular range. See [`converter`] for more
    ///   details.
    pub fn new(
        converter: C,
    ) -> SearchIndexBuilder<
        DefaultFMIndex<T, C, SuffixOrderSampledArray>,
        T,
        C,
        SuffixOrderSampledArray,
    > {
        SearchIndexBuilder {
            converter,
            get_sample: Box::new(|sa| suffix_array::sample(sa, 0)),
            _i: std::marker::PhantomData,
            _t: std::marker::PhantomData,
        }
    }
}

impl<I, T, C, S> SearchIndexBuilder<I, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    /// Make sure the index only supports the count operation.
    ///
    /// The suffix array for the locate operation will be dropped from the index.
    pub fn count_only(self) -> SearchIndexBuilder<DefaultFMIndex<T, C, ()>, T, C, ()> {
        SearchIndexBuilder {
            converter: self.converter,
            get_sample: Box::new(|_| ()),
            _i: std::marker::PhantomData,
            _t: self._t,
        }
    }

    /// Make sure the index will use RLFM-Index, which encodes the backing Wavelet Matrix using run-length encoding.
    ///
    /// The index will be more space-efficient than the FM-Index, but is slower.
    pub fn run_length_encoding(self) -> SearchIndexBuilder<RLFMIndex<T, C, S>, T, C, S> {
        SearchIndexBuilder {
            converter: self.converter,
            get_sample: self.get_sample,
            _i: std::marker::PhantomData,
            _t: self._t,
        }
    }
}

impl<I, T, C> SearchIndexBuilder<I, T, C, SuffixOrderSampledArray>
where
    I: FMIndex,
    T: Character,
    C: Converter<T>,
{
    /// Adjust the sampling level of the suffix array to use for position lookup.
    ///
    /// A sampling level of 0 means the most memory is used (a full suffix-array is
    /// retained), while looking up positions is faster. A sampling level of
    /// 1 means half the memory is used, but looking up positions is slower.
    /// Each increase in level halves the memory usage but slows down
    /// position lookup.
    pub fn level(mut self, level: usize) -> SearchIndexBuilder<I, T, C, SuffixOrderSampledArray> {
        self.get_sample = Box::new(move |sa| suffix_array::sample(sa, level));
        self
    }
}

impl<T, C, S> SearchIndexBuilder<DefaultFMIndex<T, C, S>, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    /// Build a new [SearchIndex] backed by [FMIndex].
    pub fn build(self, text: Vec<T>) -> SearchIndex<DefaultFMIndex<T, C, S>> {
        SearchIndex {
            index: DefaultFMIndex::create(text, self.converter, self.get_sample),
        }
    }
}

impl<T, C, S> SearchIndexBuilder<RLFMIndex<T, C, S>, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    /// Build a new [SearchIndex] backed by [RLFMIndex].
    pub fn build(self, text: Vec<T>) -> SearchIndex<RLFMIndex<T, C, S>> {
        SearchIndex {
            index: RLFMIndex::create(text, self.converter, self.get_sample),
        }
    }
}

/// A full-text index backed by FM-Index or its variant.
pub struct SearchIndex<I: FMIndex> {
    index: I,
}

impl<I: FMIndex> SearchIndex<I> {
    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    pub fn search<K: AsRef<[I::T]>>(&self, pattern: K) -> Search<I> {
        self.index.search(pattern)
    }
}

/// An object containing the result of a search.
///
/// This is expanded with a `locate` method if the index is
/// supplied with a sampled suffix array.
pub struct Search<'a, I: FMIndex> {
    index: &'a I,
    s: u64,
    e: u64,
    pattern: Vec<I::T>,
}

impl<'a, I> Search<'a, I>
where
    I: FMIndex,
{
    pub(crate) fn new(index: &'a I) -> Search<'a, I> {
        Search {
            index,
            s: 0,
            e: index.len::<seal::Local>(),
            pattern: vec![],
        }
    }

    /// Search in the current search result, refining it.
    ///
    /// This adds a prefix `pattern` to the existing pattern, and
    /// looks for those expanded patterns in the text.
    pub fn search<K: AsRef<[I::T]>>(&self, pattern: K) -> Self {
        let mut s = self.s;
        let mut e = self.e;
        let mut pattern = pattern.as_ref().to_vec();
        for &c in pattern.iter().rev() {
            s = self.index.lf_map2::<seal::Local>(c, s);
            e = self.index.lf_map2::<seal::Local>(c, e);
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

    #[cfg(test)]
    pub(crate) fn get_range(&self) -> (u64, u64) {
        (self.s, self.e)
    }

    /// Count the number of occurrences.
    pub fn count(&self) -> u64 {
        self.e - self.s
    }
}

impl<I> Search<'_, I>
where
    I: FMIndex + IndexWithConverter<I::T>,
{
    /// Get an iterator that goes backwards through the text, producing
    /// [`Character`].
    pub fn iter_backward(&self, i: u64) -> impl Iterator<Item = I::T> + use<'_, I> {
        let m = self.count();

        debug_assert!(m > 0, "cannot iterate from empty search result");
        debug_assert!(i < m, "{} is out of range", i);

        self.index.iter_backward::<seal::Local>(self.s + i)
    }
}

impl<I> Search<'_, I>
where
    I: FMIndex + IndexWithConverter<I::T>,
{
    /// Get an iterator that goes forwards through the text, producing
    /// [`Character`].
    pub fn iter_forward(&self, i: u64) -> impl Iterator<Item = I::T> + use<'_, I> {
        let m = self.count();

        debug_assert!(m > 0, "cannot iterate from empty search result");
        debug_assert!(i < m, "{} is out of range", i);

        self.index.iter_forward::<seal::Local>(self.s + i)
    }
}

impl<I> Search<'_, I>
where
    I: FMIndex + HasPosition,
{
    /// List the position of all occurrences.
    pub fn locate(&self) -> Vec<u64> {
        let mut results: Vec<u64> = Vec::with_capacity((self.e - self.s) as usize);
        for k in self.s..self.e {
            results.push(self.index.get_sa::<seal::Local>(k));
        }
        results
    }
}
