// This module contains the API exposed to the frontend.
//
// It consists of concrete implementations of the search index and search
// traits; one for count-only and the other supporting locate queries.
//
// This wrapper code is very verbose, so written using macros to avoid repetition.
//
// It's written around the wrapper module which provides the implementation of
// the behavior. This module only exists so we can avoid exposing implementation
// traits.

use crate::backend::HeapSize;
use crate::fm_index::FMIndexBackend;
use crate::multi_text::MultiTextFMIndexBackend;
use crate::rlfmi::RLFMIndexBackend;
use crate::suffix_array::sample::{self, SuffixOrderSampledArray};
use crate::text::TextId;
use crate::wrapper::{MatchWrapper, SearchWrapper};
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

/// Trait for searching in an index that also supports locate queries.
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

/// Trait for searching in an index that supports multiple texts.
pub trait SearchIndexWithMultiTexts<T>: SearchIndex<T> {
    /// Search for a pattern that is a suffix of a text.
    fn search_suffix<K>(&self, pattern: K) -> impl Search<T>
    where
        K: AsRef<[T]>;
}

/// The result of a search.
///
/// A search result can be refined by adding more characters to the
/// search pattern.
/// A search result contains matches, which can be iterated over.
pub trait Search<'a, T> {
    /// Associated type for matches.
    type Match: Match<'a, T>;

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
    /// Get an iterator over all matches.
    fn iter_matches(&'a self) -> impl Iterator<Item = Self::Match> + 'a;
}

/// The result of a search that also has locate support.
pub trait SearchWithLocate<'a, T>: Search<'a, T> {
    /// List the position of all occurrences.
    fn locate(&self) -> Vec<u64>;
}

/// A match in the text.
pub trait Match<'a, T> {
    /// Iterate over the characters of the match.
    fn iter_chars_forward(&self) -> impl Iterator<Item = T> + 'a;

    /// Iterate over the characters of the match in reverse.
    fn iter_chars_backward(&self) -> impl Iterator<Item = T> + 'a;
}

/// A match in the text that contains its location on the text.
pub trait MatchWithLocate<'a, T>: Match<'a, T> {
    /// Get the location of the match in the text.
    fn locate(&self) -> u64;
}

/// A match in the text that contains its text ID on the text.
pub trait MatchWithTextId<'a, T>: Match<'a, T> {
    /// Get the ID of the text that the character at the matched position belongs to.
    fn text_id(&self) -> TextId;
}

/// FMIndex, count only.
///
/// The FM-Index is both a search index as well as compact representation of
/// the text.
pub struct FMIndex<T: Character, C: Converter<T>>(SearchIndexWrapper<FMIndexBackend<T, C, ()>>);
/// Search result for FMIndex, count only.
pub struct FMIndexSearch<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, FMIndexBackend<T, C, ()>>,
);
/// Match in the text for FMIndex.
pub struct FMIndexMatch<'a, T: Character, C: Converter<T>>(
    MatchWrapper<'a, FMIndexBackend<T, C, ()>>,
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
/// Match in the text for FMIndex with locate support.
pub struct FMIndexMatchWithLocate<'a, T: Character, C: Converter<T>>(
    MatchWrapper<'a, FMIndexBackend<T, C, SuffixOrderSampledArray>>,
);

/// RLFMIndex, count only.
///
/// This is a version of the FM-Index that uses less space, but is also less efficient.
pub struct RLFMIndex<T: Character, C: Converter<T>>(SearchIndexWrapper<RLFMIndexBackend<T, C, ()>>);
/// Search result for RLFMIndex, count only.
pub struct RLFMIndexSearch<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, RLFMIndexBackend<T, C, ()>>,
);
/// Match in the text for RLFMIndex.
pub struct RLFMIndexMatch<'a, T: Character, C: Converter<T>>(
    MatchWrapper<'a, RLFMIndexBackend<T, C, ()>>,
);

/// RLFMIndex with locate support.
///
/// This is a version of the FM-Index that uses less space, but is also less efficient.
/// It uses additional storage to support locate queries.
pub struct RLFMIndexWithLocate<T: Character, C: Converter<T>>(
    SearchIndexWrapper<RLFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);
/// Search result for RLFMIndex with locate support.
pub struct RLFMIndexSearchWithLocate<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, RLFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);
/// Match in the text for RLFMIndex with locate support.
pub struct RLFMIndexMatchWithLocate<'a, T: Character, C: Converter<T>>(
    MatchWrapper<'a, RLFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);

/// MultiText index, count only.
///
/// This is a multi-text version of the FM-Index. It allows \0 separated strings.
pub struct MultiTextFMIndex<T: Character, C: Converter<T>>(
    SearchIndexWrapper<MultiTextFMIndexBackend<T, C, ()>>,
);
/// Search result for MultiText index, count only.
pub struct MultiTextFMIndexSearch<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, MultiTextFMIndexBackend<T, C, ()>>,
);
/// Match in the text for MultiText index.
pub struct MultiTextFMIndexMatch<'a, T: Character, C: Converter<T>>(
    MatchWrapper<'a, MultiTextFMIndexBackend<T, C, ()>>,
);

/// MultiText index with locate support.
///
/// This is a multi-text version of the FM-Index. It allows \0 separated strings.
/// It uses additional storage to support locate queries.
pub struct MultiTextFMIndexWithLocate<T: Character, C: Converter<T>>(
    SearchIndexWrapper<MultiTextFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);
/// Search result for MultiText index with locate support.
pub struct MultiTextFMIndexSearchWithLocate<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, MultiTextFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);
/// Match in the text for MultiText index with locate support.
pub struct MultiTextFMIndexMatchWithLocate<'a, T: Character, C: Converter<T>>(
    MatchWrapper<'a, MultiTextFMIndexBackend<T, C, SuffixOrderSampledArray>>,
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

macro_rules! impl_search_index_with_multi_texts {
    ($t:ty, $s:ident, $st:ty) => {
        impl<T: Character, C: Converter<T>> SearchIndexWithMultiTexts<T> for $t {
            fn search_suffix<K>(&self, pattern: K) -> impl Search<T>
            where
                K: AsRef<[T]>,
            {
                $s(self.0.search_suffix(pattern))
            }
        }

        // inherent
        impl<T: Character, C: Converter<T>> $t {
            /// Search for a pattern in the text.
            pub fn search_suffix<K>(&self, pattern: K) -> $st
            where
                K: AsRef<[T]>,
            {
                $s(self.0.search_suffix(pattern))
            }
        }
    };
}

macro_rules! impl_search {
    ($t:ty, $m:ident, $mt:ty) => {
        impl<'a, T: Character, C: Converter<T>> Search<'a, T> for $t {
            type Match = $mt;

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

            fn iter_matches(&'a self) -> impl Iterator<Item = Self::Match> + 'a {
                self.0.iter_matches().map(|m| $m(m))
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

macro_rules! impl_match {
    ($t:ty) => {
        impl<'a, T: Character, C: Converter<T>> Match<'a, T> for $t {
            fn iter_chars_forward(&self) -> impl Iterator<Item = T> + 'a {
                self.0.iter_chars_forward()
            }

            fn iter_chars_backward(&self) -> impl Iterator<Item = T> + 'a {
                self.0.iter_chars_backward()
            }
        }
    };
}

macro_rules! impl_match_locate {
    ($t:ty) => {
        impl<'a, T: Character, C: Converter<T>> MatchWithLocate<'a, T> for $t {
            fn locate(&self) -> u64 {
                self.0.locate()
            }
        }
    };
}

macro_rules! impl_match_text_id {
    ($t:ty) => {
        impl<'a, T: Character, C: Converter<T>> MatchWithTextId<'a, T> for $t {
            fn text_id(&self) -> TextId {
                self.0.text_id()
            }
        }
    };
}

impl_search_index!(FMIndex<T, C>, FMIndexSearch, FMIndexSearch<T, C>);
impl_search!(
    FMIndexSearch<'a, T, C>,
    FMIndexMatch,
    FMIndexMatch<'a, T, C>
);
impl_match!(FMIndexMatch<'a, T, C>);

impl_search_index_with_locate!(FMIndexWithLocate<T, C>, FMIndexSearchWithLocate, FMIndexSearchWithLocate<T, C>);
impl_search!(
    FMIndexSearchWithLocate<'a, T, C>,
    FMIndexMatchWithLocate,
    FMIndexMatchWithLocate<'a, T, C>
);
impl_search_locate!(FMIndexSearchWithLocate<'a, T, C>);
impl_match!(FMIndexMatchWithLocate<'a, T, C>);
impl_match_locate!(FMIndexMatchWithLocate<'a, T, C>);

impl_search_index!(RLFMIndex<T, C>, RLFMIndexSearch, RLFMIndexSearch<T, C>);
impl_search!(
    RLFMIndexSearch<'a, T, C>,
    RLFMIndexMatch,
    RLFMIndexMatch<'a, T, C>
);
impl_match!(RLFMIndexMatch<'a, T, C>);

impl_search_index_with_locate!(RLFMIndexWithLocate<T, C>, RLFMIndexSearchWithLocate, RLFMIndexSearchWithLocate<T, C>);
impl_search!(
    RLFMIndexSearchWithLocate<'a, T, C>,
    RLFMIndexMatchWithLocate,
    RLFMIndexMatchWithLocate<'a, T, C>
);
impl_search_locate!(RLFMIndexSearchWithLocate<'a, T, C>);
impl_match!(RLFMIndexMatchWithLocate<'a, T, C>);
impl_match_locate!(RLFMIndexMatchWithLocate<'a, T, C>);

impl_search_index!(MultiTextFMIndex<T, C>, MultiTextFMIndexSearch, MultiTextFMIndexSearch<T, C>);
impl_search_index_with_multi_texts!(MultiTextFMIndex<T, C>, MultiTextFMIndexSearch, MultiTextFMIndexSearch<T, C>);
impl_search!(
    MultiTextFMIndexSearch<'a, T, C>,
    MultiTextFMIndexMatch,
    MultiTextFMIndexMatch<'a, T, C>
);
impl_match!(MultiTextFMIndexMatch<'a, T, C>);

impl_search_index_with_locate!(MultiTextFMIndexWithLocate<T, C>, MultiTextFMIndexSearchWithLocate, MultiTextFMIndexSearchWithLocate<T, C>);
impl_search_index_with_multi_texts!(MultiTextFMIndexWithLocate<T, C>, MultiTextFMIndexSearchWithLocate, MultiTextFMIndexSearchWithLocate<T, C>);
impl_search!(
    MultiTextFMIndexSearchWithLocate<'a, T, C>,
    MultiTextFMIndexMatchWithLocate,
    MultiTextFMIndexMatchWithLocate<'a, T, C>
);
impl_search_locate!(MultiTextFMIndexSearchWithLocate<'a, T, C>);
impl_match!(MultiTextFMIndexMatchWithLocate<'a, T, C>);
impl_match_locate!(MultiTextFMIndexMatchWithLocate<'a, T, C>);
impl_match_text_id!(MultiTextFMIndexMatchWithLocate<'a, T, C>);
