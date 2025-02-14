use crate::backend::HeapSize;
use crate::fm_index::FMIndexBackend;
use crate::multi_text::MultiTextFMIndexBackend;
use crate::rlfmi::RLFMIndexBackend;
use crate::suffix_array::sample::{self, SuffixOrderSampledArray};
use crate::wrapper::SearchWrapper;
use crate::{converter::Converter, wrapper::SearchIndexWrapper, Character};

/// Trait for searching in an index.
///
/// You can use this to search in an index generically.
pub trait SearchIndex<T> {
    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    fn search<K>(&self, pattern: K) -> impl Search<T>
    where
        K: AsRef<[T]>;

    /// The size of the text in the index
    ///
    /// Note that this includes an ending \0 (terminator) character
    /// so will be one more than the length of the text.
    fn len(&self) -> u64;
}

/// Trait for searching in an index that supports locate queries.
///
/// You can use this to search in an index generically.
pub trait SearchIndexWithLocate<T>: SearchIndex<T> {
    /// Search for a pattern in the text.
    ///
    /// Return a [`SearchWithLocate`] object with information about the search
    /// result, which also supports locate queries.
    fn search<K>(&self, pattern: K) -> impl SearchWithLocate<T>
    where
        K: AsRef<[T]>;
}

/// The result of a search.
pub trait Search<'a, T> {
    /// Search in the current search result, refining it.
    ///
    /// This adds a prefix `pattern` to the existing pattern, and
    /// looks for those expanded patterns in the text.
    fn search<K: AsRef<[T]>>(&self, pattern: K) -> Self;
    /// Count the number of occurrences.
    fn count(&self) -> u64;
    /// Get an iterator that goes backwards through the text, producing
    /// [`Character`].
    fn iter_backward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a;
    /// Get an iterator that goes forwards through the text, producing
    /// [`Character`].
    fn iter_forward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a;
}

/// The result of a search with locate support.
pub trait SearchWithLocate<'a, T>: Search<'a, T> {
    /// List the position of all occurrences.
    fn locate(&self) -> Vec<u64>;
}

/// FMIndex without locate support.
///
/// The FM-Index is both a search index as well as compact representation of
/// the text.
pub struct FMIndex<T: Character, C: Converter<T>>(SearchIndexWrapper<FMIndexBackend<T, C, ()>>);
/// Search result for FMIndex without locate support.
pub struct FMIndexSearch<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, FMIndexBackend<T, C, ()>>,
);

/// FMIndex with locate support.
///
/// This is an FM-Index which uses additional storage to support locate queries.
pub struct FMIndexWithLocate<T: Character, C: Converter<T>>(
    SearchIndexWrapper<FMIndexBackend<T, C, SuffixOrderSampledArray>>,
);
/// Search result for FMIndex with locate support.
pub struct FMIndexSearchWithLocate<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, FMIndexBackend<T, C, SuffixOrderSampledArray>>,
);

/// RLFMIndex without locate support.
///
/// This is a reduced space version of the FM-Index, but it's less efficient.
pub struct RLFMIndex<T: Character, C: Converter<T>>(SearchIndexWrapper<RLFMIndexBackend<T, C, ()>>);
/// Search result for RLFMIndex without locate support.
pub struct RLFMIndexSearch<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, RLFMIndexBackend<T, C, ()>>,
);

/// RLFMIndex with locate support.
///
/// This is a reduced space version of the FM-Index, but it's less efficient. It
/// uses additional storage to support locate queries.
pub struct RLFMIndexWithLocate<T: Character, C: Converter<T>>(
    SearchIndexWrapper<RLFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);
/// Search result for RLFMIndex with locate support.
pub struct RLFMIndexSearchWithLocate<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, RLFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);

/// MultiText index without locate support.
///
/// This is a multi-text version of the FM-Index.
pub struct MultiTextFMIndex<T: Character, C: Converter<T>>(
    SearchIndexWrapper<MultiTextFMIndexBackend<T, C, ()>>,
);
/// Search result for MultiText index without locate support.
pub struct MultiTextFMIndexSearch<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, MultiTextFMIndexBackend<T, C, ()>>,
);

/// MultiText index with locate support.
pub struct MultiTextFMIndexWithLocate<T: Character, C: Converter<T>>(
    SearchIndexWrapper<MultiTextFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);
/// Search result for MultiText index with locate support.
pub struct MultiTextFMIndexSearchWithLocate<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, MultiTextFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);

impl<T: Character, C: Converter<T>> FMIndex<T, C> {
    /// Create a new FMIndex without locate support.
    pub fn new(text: Vec<T>, converter: C) -> Self {
        FMIndex(SearchIndexWrapper::new(FMIndexBackend::new(
            text,
            converter,
            |_| (),
        )))
    }
}

impl<T: Character, C: Converter<T>> FMIndexWithLocate<T, C> {
    /// Create a new FMIndex with locate support.
    ///
    /// The level argument controls the sampling rate used. Higher levels use
    /// less storage, at the cost of performance of locate queries. A level of
    /// 0 means no sampling, and a level of 1 means half of the suffix array is
    /// sampled, a level of 2 means a quarter of the suffix array is sampled,
    /// and so on.
    pub fn new(text: Vec<T>, converter: C, level: usize) -> Self {
        FMIndexWithLocate(SearchIndexWrapper::new(FMIndexBackend::new(
            text,
            converter,
            |sa| sample::sample(sa, level),
        )))
    }
}

impl<T: Character, C: Converter<T>> RLFMIndex<T, C> {
    /// Create a new RLFMIndex without locate support.
    pub fn new(text: Vec<T>, converter: C) -> Self {
        RLFMIndex(SearchIndexWrapper::new(RLFMIndexBackend::new(
            text,
            converter,
            |_| (),
        )))
    }
}

impl<T: Character, C: Converter<T>> RLFMIndexWithLocate<T, C> {
    /// Create a new RLFMIndex with locate support.
    ///
    /// The level argument controls the sampling rate used. Higher levels use
    /// less storage, at the cost of performance of locate queries. A level of
    /// 0 means no sampling, and a level of 1 means half of the suffix array is
    /// sampled, a level of 2 means a quarter of the suffix array is sampled,
    /// and so on.
    pub fn new(text: Vec<T>, converter: C, level: usize) -> Self {
        RLFMIndexWithLocate(SearchIndexWrapper::new(RLFMIndexBackend::new(
            text,
            converter,
            |sa| sample::sample(sa, level),
        )))
    }
}

impl<T: Character, C: Converter<T>> MultiTextFMIndex<T, C> {
    /// Create a new MultiTextFMIndex without locate support.
    pub fn new(text: Vec<T>, converter: C) -> Self {
        MultiTextFMIndex(SearchIndexWrapper::new(MultiTextFMIndexBackend::new(
            text,
            converter,
            |_| (),
        )))
    }
}

impl<T: Character, C: Converter<T>> MultiTextFMIndexWithLocate<T, C> {
    /// Create a new MultiTextFMIndex with locate support.
    ///
    /// The level argument controls the sampling rate used. Higher levels use
    /// less storage, at the cost of performance of locate queries. A level of
    /// 0 means no sampling, and a level of 1 means half of the suffix array is
    /// sampled, a level of 2 means a quarter of the suffix array is sampled,
    /// and so on.
    pub fn new(text: Vec<T>, converter: C, level: usize) -> Self {
        MultiTextFMIndexWithLocate(SearchIndexWrapper::new(MultiTextFMIndexBackend::new(
            text,
            converter,
            |sa| sample::sample(sa, level),
        )))
    }
}

macro_rules! impl_search_index {
    ($t:ty, $s:ident, $st:ty) => {
        impl<T: Character, C: Converter<T>> SearchIndex<T> for $t {
            fn search<K>(&self, pattern: K) -> impl Search<T>
            where
                K: AsRef<[T]>,
            {
                $s(self.0.search(pattern))
            }

            fn len(&self) -> u64 {
                self.0.len()
            }
        }
        impl<T: Character, C: Converter<T>> HeapSize for $t {
            fn heap_size(&self) -> usize {
                self.0.heap_size()
            }
        }
        // inherent
        impl<T: Character, C: Converter<T>> $t {
            /// Search for a pattern in the text.
            pub fn search<K>(&self, pattern: K) -> $st
            where
                K: AsRef<[T]>,
            {
                $s(self.0.search(pattern))
            }
            /// The size of the text in the index
            pub fn len(&self) -> u64 {
                SearchIndex::len(self)
            }
        }
    };
}

macro_rules! impl_search_index_with_locate {
    ($t:ty, $s:ident, $st:ty) => {
        impl<T: Character, C: Converter<T>> SearchIndex<T> for $t {
            fn search<K>(&self, pattern: K) -> impl Search<T>
            where
                K: AsRef<[T]>,
            {
                $s(self.0.search(pattern))
            }

            fn len(&self) -> u64 {
                self.0.len()
            }
        }
        impl<T: Character, C: Converter<T>> SearchIndexWithLocate<T> for $t {
            fn search<K>(&self, pattern: K) -> impl SearchWithLocate<T>
            where
                K: AsRef<[T]>,
            {
                $s(self.0.search(pattern))
            }
        }
        impl<T: Character, C: Converter<T>> HeapSize for $t {
            fn heap_size(&self) -> usize {
                self.0.heap_size()
            }
        }
        // inherent
        impl<T: Character, C: Converter<T>> $t {
            /// Search for a pattern in the text.
            pub fn search<K>(&self, pattern: K) -> $st
            where
                K: AsRef<[T]>,
            {
                $s(self.0.search(pattern))
            }
            /// The size of the text in the index
            pub fn len(&self) -> u64 {
                SearchIndex::len(self)
            }
        }
    };
}

macro_rules! impl_search {
    ($t:ty) => {
        impl<'a, T: Character, C: Converter<T>> Search<'a, T> for $t {
            fn search<K>(&self, pattern: K) -> Self
            where
                K: AsRef<[T]>,
            {
                Self(self.0.search(pattern))
            }

            fn count(&self) -> u64 {
                self.0.count()
            }

            fn iter_backward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a {
                self.0.iter_backward(i)
            }

            fn iter_forward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a {
                self.0.iter_forward(i)
            }
        }
        // inherent
        impl<'a, T: Character, C: Converter<T>> $t {
            /// Search in the current search result, refining it.
            ///
            /// This adds a prefix `pattern` to the existing pattern, and
            /// looks for those expanded patterns in the text.
            pub fn search<K>(&self, pattern: K) -> Self
            where
                K: AsRef<[T]>,
            {
                Search::search(self, pattern)
            }

            /// Count the number of occurrences.
            pub fn count(&self) -> u64 {
                Search::count(self)
            }

            /// Get an iterator that goes backwards through the text, producing
            /// [`Character`].
            pub fn iter_backward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a {
                Search::iter_backward(self, i)
            }

            /// Get an iterator that goes forwards through the text, producing
            /// [`Character`].
            pub fn iter_forward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a {
                Search::iter_forward(self, i)
            }
        }
    };
}

macro_rules! impl_search_locate {
    ($t:ty) => {
        impl<'a, T: Character, C: Converter<T>> SearchWithLocate<'a, T> for $t {
            fn locate(&self) -> Vec<u64> {
                self.0.locate()
            }
        }
        // inherent
        impl<'a, T: Character, C: Converter<T>> $t {
            /// List the position of all occurrences.
            pub fn locate(&self) -> Vec<u64> {
                SearchWithLocate::locate(self)
            }
        }
    };
}

impl_search_index!(FMIndex<T, C>, FMIndexSearch, FMIndexSearch<T, C>);
impl_search!(FMIndexSearch<'a, T, C>);

impl_search_index_with_locate!(FMIndexWithLocate<T, C>, FMIndexSearchWithLocate, FMIndexSearchWithLocate<T, C>);
impl_search!(FMIndexSearchWithLocate<'a, T, C>);
impl_search_locate!(FMIndexSearchWithLocate<'a, T, C>);

impl_search_index!(RLFMIndex<T, C>, RLFMIndexSearch, RLFMIndexSearch<T, C>);
impl_search!(RLFMIndexSearch<'a, T, C>);

impl_search_index_with_locate!(RLFMIndexWithLocate<T, C>, RLFMIndexSearchWithLocate, RLFMIndexSearchWithLocate<T, C>);
impl_search!(RLFMIndexSearchWithLocate<'a, T, C>);
impl_search_locate!(RLFMIndexSearchWithLocate<'a, T, C>);

impl_search_index!(MultiTextFMIndex<T, C>, MultiTextFMIndexSearch, MultiTextFMIndexSearch<T, C>);
impl_search!(MultiTextFMIndexSearch<'a, T, C>);

impl_search_index_with_locate!(MultiTextFMIndexWithLocate<T, C>, MultiTextFMIndexSearchWithLocate, MultiTextFMIndexSearchWithLocate<T, C>);
impl_search!(MultiTextFMIndexSearchWithLocate<'a, T, C>);
impl_search_locate!(MultiTextFMIndexSearchWithLocate<'a, T, C>);
