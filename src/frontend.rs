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
use crate::character::Character;
use crate::fm_index::FMIndexBackend;
use crate::multi_text::MultiTextFMIndexBackend;
use crate::rlfmi::RLFMIndexBackend;
use crate::suffix_array::sample::{self, SuffixOrderSampledArray};
use crate::text::{Text, TextId};
use crate::wrapper::{MatchWrapper, SearchIndexWrapper, SearchWrapper};

/// Trait for searching in an index.
///
/// You can use this to search in an index generically.
pub trait SearchIndex<C> {
    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    fn search<K>(&self, pattern: K) -> impl Search<C>
    where
        K: AsRef<[C]>;

    /// The size of the text in the index
    ///
    /// Note that this includes an ending \0 (terminator) character
    /// so will be one more than the length of the text.
    fn len(&self) -> usize;
}

/// Trait for searching in an index that supports multiple texts.
pub trait SearchIndexWithMultiTexts<C>: SearchIndex<C> {
    /// Search for a pattern that is a prefix of a text.
    fn search_prefix<K>(&self, pattern: K) -> impl Search<C>
    where
        K: AsRef<[C]>;

    /// Search for a pattern that is a suffix of a text.
    fn search_suffix<K>(&self, pattern: K) -> impl Search<C>
    where
        K: AsRef<[C]>;

    /// Search for a pattern that is an exact match of a text.
    fn search_exact<K>(&self, pattern: K) -> impl Search<C>
    where
        K: AsRef<[C]>;
}

/// The result of a search.
///
/// A search result can be refined by adding more characters to the
/// search pattern.
/// A search result contains matches, which can be iterated over.
pub trait Search<'a, C> {
    /// Associated type for matches.
    type Match: Match<'a, C>;

    /// Search in the current search result, refining it.
    ///
    /// This adds a prefix `pattern` to the existing pattern, and
    /// looks for those expanded patterns in the text.
    fn search<K: AsRef<[C]>>(&self, pattern: K) -> Self;
    /// Count the number of occurrences.
    fn count(&self) -> usize;
    /// Get an iterator over all matches.
    fn iter_matches(&'a self) -> impl Iterator<Item = Self::Match> + 'a;
}

/// The result of a search that also has locate support.
pub trait SearchWithLocate<'a, C>: Search<'a, C> {
    /// List the position of all occurrences.
    fn locate(&self) -> Vec<u64>;
}

/// A match in the text.
pub trait Match<'a, C> {
    /// Iterate over the characters of the match.
    fn iter_chars_forward(&self) -> impl Iterator<Item = C> + 'a;

    /// Iterate over the characters of the match in reverse.
    fn iter_chars_backward(&self) -> impl Iterator<Item = C> + 'a;
}

/// A match in the text that contains its location on the text.
pub trait MatchWithLocate<'a, C>: Match<'a, C> {
    /// Get the location of the match in the text.
    fn locate(&self) -> usize;
}

/// A match in the text that contains its text ID on the text.
pub trait MatchWithTextId<'a, C>: Match<'a, C> {
    /// Get the ID of the text that the character at the matched position belongs to.
    fn text_id(&self) -> TextId;
}

/// FMIndex, count only.
///
/// The FM-Index is both a search index as well as compact representation of
/// the text.
pub struct FMIndex<C: Character>(SearchIndexWrapper<FMIndexBackend<C, ()>>);
/// Search result for FMIndex, count only.
pub struct FMIndexSearch<'a, C: Character>(SearchWrapper<'a, FMIndexBackend<C, ()>>);
/// Match in the text for FMIndex.
pub struct FMIndexMatch<'a, C: Character>(MatchWrapper<'a, FMIndexBackend<C, ()>>);

/// FMIndex with locate support.
///
/// This is an FM-Index which uses additional storage to support locate queries.
pub struct FMIndexWithLocate<C: Character>(
    SearchIndexWrapper<FMIndexBackend<C, SuffixOrderSampledArray>>,
);
/// Search result for FMIndex with locate support.
pub struct FMIndexSearchWithLocate<'a, C: Character>(
    SearchWrapper<'a, FMIndexBackend<C, SuffixOrderSampledArray>>,
);
/// Match in the text for FMIndex with locate support.
pub struct FMIndexMatchWithLocate<'a, C: Character>(
    MatchWrapper<'a, FMIndexBackend<C, SuffixOrderSampledArray>>,
);

/// RLFMIndex, count only.
///
/// This is a version of the FM-Index that uses less space, but is also less efficient.
pub struct RLFMIndex<C: Character>(SearchIndexWrapper<RLFMIndexBackend<C, ()>>);
/// Search result for RLFMIndex, count only.
pub struct RLFMIndexSearch<'a, C: Character>(SearchWrapper<'a, RLFMIndexBackend<C, ()>>);
/// Match in the text for RLFMIndex.
pub struct RLFMIndexMatch<'a, C: Character>(MatchWrapper<'a, RLFMIndexBackend<C, ()>>);

/// RLFMIndex with locate support.
///
/// This is a version of the FM-Index that uses less space, but is also less efficient.
/// It uses additional storage to support locate queries.
pub struct RLFMIndexWithLocate<C: Character>(
    SearchIndexWrapper<RLFMIndexBackend<C, SuffixOrderSampledArray>>,
);
/// Search result for RLFMIndex with locate support.
pub struct RLFMIndexSearchWithLocate<'a, C: Character>(
    SearchWrapper<'a, RLFMIndexBackend<C, SuffixOrderSampledArray>>,
);
/// Match in the text for RLFMIndex with locate support.
pub struct RLFMIndexMatchWithLocate<'a, C: Character>(
    MatchWrapper<'a, RLFMIndexBackend<C, SuffixOrderSampledArray>>,
);

/// MultiText index, count only.
///
/// This is a multi-text version of the FM-Index. It allows \0 separated strings.
pub struct MultiTextFMIndex<C: Character>(SearchIndexWrapper<MultiTextFMIndexBackend<C, ()>>);
/// Search result for MultiText index, count only.
pub struct MultiTextFMIndexSearch<'a, C: Character>(
    SearchWrapper<'a, MultiTextFMIndexBackend<C, ()>>,
);
/// Match in the text for MultiText index.
pub struct MultiTextFMIndexMatch<'a, C: Character>(
    MatchWrapper<'a, MultiTextFMIndexBackend<C, ()>>,
);

/// MultiText index with locate support.
///
/// This is a multi-text version of the FM-Index. It allows \0 separated strings.
/// It uses additional storage to support locate queries.
pub struct MultiTextFMIndexWithLocate<C: Character>(
    SearchIndexWrapper<MultiTextFMIndexBackend<C, SuffixOrderSampledArray>>,
);
/// Search result for MultiText index with locate support.
pub struct MultiTextFMIndexSearchWithLocate<'a, C: Character>(
    SearchWrapper<'a, MultiTextFMIndexBackend<C, SuffixOrderSampledArray>>,
);
/// Match in the text for MultiText index with locate support.
pub struct MultiTextFMIndexMatchWithLocate<'a, C: Character>(
    MatchWrapper<'a, MultiTextFMIndexBackend<C, SuffixOrderSampledArray>>,
);

impl<C: Character> FMIndex<C> {
    /// Create a new FMIndex without locate support.
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>) -> Self {
        FMIndex(SearchIndexWrapper::new(FMIndexBackend::new(text, |_| ())))
    }
}

impl<C: Character> FMIndexWithLocate<C> {
    /// Create a new FMIndex with locate support.
    ///
    /// The level argument controls the sampling rate used. Higher levels use
    /// less storage, at the cost of performance of locate queries. A level of
    /// 0 means no sampling, and a level of 1 means half of the suffix array is
    /// sampled, a level of 2 means a quarter of the suffix array is sampled,
    /// and so on.
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>, level: usize) -> Self {
        FMIndexWithLocate(SearchIndexWrapper::new(FMIndexBackend::new(text, |sa| {
            sample::sample(sa, level)
        })))
    }
}

impl<C: Character> RLFMIndex<C> {
    /// Create a new RLFMIndex without locate support.
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>) -> Self {
        RLFMIndex(SearchIndexWrapper::new(RLFMIndexBackend::new(text, |_| ())))
    }
}

impl<C: Character> RLFMIndexWithLocate<C> {
    /// Create a new RLFMIndex with locate support.
    ///
    /// The level argument controls the sampling rate used. Higher levels use
    /// less storage, at the cost of performance of locate queries. A level of
    /// 0 means no sampling, and a level of 1 means half of the suffix array is
    /// sampled, a level of 2 means a quarter of the suffix array is sampled,
    /// and so on.
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>, level: usize) -> Self {
        RLFMIndexWithLocate(SearchIndexWrapper::new(RLFMIndexBackend::new(text, |sa| {
            sample::sample(sa, level)
        })))
    }
}

impl<C: Character> MultiTextFMIndex<C> {
    /// Create a new MultiTextFMIndex without locate support.
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>) -> Self {
        MultiTextFMIndex(SearchIndexWrapper::new(MultiTextFMIndexBackend::new(
            text,
            |_| (),
        )))
    }
}

impl<C: Character> MultiTextFMIndexWithLocate<C> {
    /// Create a new MultiTextFMIndex with locate support.
    ///
    /// The level argument controls the sampling rate used. Higher levels use
    /// less storage, at the cost of performance of locate queries. A level of
    /// 0 means no sampling, and a level of 1 means half of the suffix array is
    /// sampled, a level of 2 means a quarter of the suffix array is sampled,
    /// and so on.
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>, level: usize) -> Self {
        MultiTextFMIndexWithLocate(SearchIndexWrapper::new(MultiTextFMIndexBackend::new(
            text,
            |sa| sample::sample(sa, level),
        )))
    }
}

macro_rules! impl_search_index {
    ($t:ty, $s:ident, $st:ty) => {
        impl<C: Character> SearchIndex<C> for $t {
            fn search<K>(&self, pattern: K) -> impl Search<C>
            where
                K: AsRef<[C]>,
            {
                $s(self.0.search(pattern))
            }

            fn len(&self) -> usize {
                self.0.len()
            }
        }
        impl<C: Character> HeapSize for $t {
            fn heap_size(&self) -> usize {
                self.0.heap_size()
            }
        }
        // inherent
        impl<C: Character> $t {
            /// Search for a pattern in the text.
            pub fn search<K>(&self, pattern: K) -> $st
            where
                K: AsRef<[C]>,
            {
                $s(self.0.search(pattern))
            }
            /// The size of the text in the index
            pub fn len(&self) -> usize {
                SearchIndex::len(self)
            }
        }
    };
}

macro_rules! impl_search_index_with_locate {
    ($t:ty, $s:ident, $st:ty) => {
        impl<C: Character> SearchIndex<C> for $t {
            fn search<K>(&self, pattern: K) -> impl Search<C>
            where
                K: AsRef<[C]>,
            {
                $s(self.0.search(pattern))
            }

            fn len(&self) -> usize {
                self.0.len()
            }
        }
        impl<C: Character> HeapSize for $t {
            fn heap_size(&self) -> usize {
                self.0.heap_size()
            }
        }
        // inherent
        impl<C: Character> $t {
            /// Search for a pattern in the text.
            pub fn search<K>(&self, pattern: K) -> $st
            where
                K: AsRef<[C]>,
            {
                $s(self.0.search(pattern))
            }
            /// The size of the text in the index
            pub fn len(&self) -> usize {
                SearchIndex::len(self)
            }
        }
    };
}

macro_rules! impl_search_index_with_multi_texts {
    ($t:ty, $s:ident, $st:ty) => {
        impl<C: Character> SearchIndexWithMultiTexts<C> for $t {
            fn search_prefix<K>(&self, pattern: K) -> impl Search<C>
            where
                K: AsRef<[C]>,
            {
                $s(self.0.search_prefix(pattern))
            }

            fn search_suffix<K>(&self, pattern: K) -> impl Search<C>
            where
                K: AsRef<[C]>,
            {
                $s(self.0.search_suffix(pattern))
            }

            fn search_exact<K>(&self, pattern: K) -> impl Search<C>
            where
                K: AsRef<[C]>,
            {
                $s(self.0.search_exact(pattern))
            }
        }

        // inherent
        impl<C: Character> $t {
            /// Search for a pattern that is a prefix of a text.
            pub fn search_prefix<K>(&self, pattern: K) -> $st
            where
                K: AsRef<[C]>,
            {
                $s(self.0.search_prefix(pattern))
            }

            /// Search for a pattern that is a suffix of a text.
            pub fn search_suffix<K>(&self, pattern: K) -> $st
            where
                K: AsRef<[C]>,
            {
                $s(self.0.search_suffix(pattern))
            }

            /// Search for a pattern that is an exact match of a text.
            pub fn search_exact<K>(&self, pattern: K) -> $st
            where
                K: AsRef<[C]>,
            {
                $s(self.0.search_exact(pattern))
            }
        }
    };
}

macro_rules! impl_search {
    ($t:ty, $m:ident, $mt:ty) => {
        impl<'a, C: Character> Search<'a, C> for $t {
            type Match = $mt;

            fn search<K>(&self, pattern: K) -> Self
            where
                K: AsRef<[C]>,
            {
                Self(self.0.search(pattern))
            }

            fn count(&self) -> usize {
                self.0.count()
            }

            fn iter_matches(&'a self) -> impl Iterator<Item = Self::Match> + 'a {
                self.0.iter_matches().map(|m| $m(m))
            }
        }
        // inherent
        impl<'a, C: Character> $t {
            /// Search in the current search result, refining it.
            ///
            /// This adds a prefix `pattern` to the existing pattern, and
            /// looks for those expanded patterns in the text.
            pub fn search<K>(&self, pattern: K) -> Self
            where
                K: AsRef<[C]>,
            {
                Search::search(self, pattern)
            }

            /// Count the number of occurrences.
            pub fn count(&self) -> usize {
                Search::count(self)
            }
        }
    };
}

macro_rules! impl_match {
    ($t:ty) => {
        impl<'a, C: Character> Match<'a, C> for $t {
            fn iter_chars_forward(&self) -> impl Iterator<Item = C> + 'a {
                self.0.iter_chars_forward()
            }

            fn iter_chars_backward(&self) -> impl Iterator<Item = C> + 'a {
                self.0.iter_chars_backward()
            }
        }
    };
}

macro_rules! impl_match_locate {
    ($t:ty) => {
        impl<'a, C: Character> MatchWithLocate<'a, C> for $t {
            fn locate(&self) -> usize {
                self.0.locate()
            }
        }
    };
}

macro_rules! impl_match_text_id {
    ($t:ty) => {
        impl<'a, C: Character> MatchWithTextId<'a, C> for $t {
            fn text_id(&self) -> TextId {
                self.0.text_id()
            }
        }
    };
}

impl_search_index!(FMIndex<C>, FMIndexSearch, FMIndexSearch<C>);
impl_search!(FMIndexSearch<'a, C>, FMIndexMatch, FMIndexMatch<'a, C>);
impl_match!(FMIndexMatch<'a, C>);

impl_search_index_with_locate!(
    FMIndexWithLocate<C>,
    FMIndexSearchWithLocate,
    FMIndexSearchWithLocate<C>
);
impl_search!(
    FMIndexSearchWithLocate<'a, C>,
    FMIndexMatchWithLocate,
    FMIndexMatchWithLocate<'a, C>
);
impl_match!(FMIndexMatchWithLocate<'a, C>);
impl_match_locate!(FMIndexMatchWithLocate<'a, C>);

impl_search_index!(RLFMIndex<C>, RLFMIndexSearch, RLFMIndexSearch<C>);
impl_search!(
    RLFMIndexSearch<'a, C>,
    RLFMIndexMatch,
    RLFMIndexMatch<'a, C>
);
impl_match!(RLFMIndexMatch<'a, C>);

impl_search_index_with_locate!(
    RLFMIndexWithLocate<C>,
    RLFMIndexSearchWithLocate,
    RLFMIndexSearchWithLocate<C>
);
impl_search!(
    RLFMIndexSearchWithLocate<'a, C>,
    RLFMIndexMatchWithLocate,
    RLFMIndexMatchWithLocate<'a, C>
);
impl_match!(RLFMIndexMatchWithLocate<'a, C>);
impl_match_locate!(RLFMIndexMatchWithLocate<'a, C>);

impl_search_index!(
    MultiTextFMIndex<C>,
    MultiTextFMIndexSearch,
    MultiTextFMIndexSearch<C>
);
impl_search_index_with_multi_texts!(
    MultiTextFMIndex<C>,
    MultiTextFMIndexSearch,
    MultiTextFMIndexSearch<C>
);
impl_search!(
    MultiTextFMIndexSearch<'a, C>,
    MultiTextFMIndexMatch,
    MultiTextFMIndexMatch<'a, C>
);
impl_match!(MultiTextFMIndexMatch<'a, C>);

impl_search_index_with_locate!(
    MultiTextFMIndexWithLocate<C>,
    MultiTextFMIndexSearchWithLocate,
    MultiTextFMIndexSearchWithLocate<C>
);
impl_search_index_with_multi_texts!(
    MultiTextFMIndexWithLocate<C>,
    MultiTextFMIndexSearchWithLocate,
    MultiTextFMIndexSearchWithLocate<C>
);
impl_search!(
    MultiTextFMIndexSearchWithLocate<'a, C>,
    MultiTextFMIndexMatchWithLocate,
    MultiTextFMIndexMatchWithLocate<'a, C>
);
impl_match!(MultiTextFMIndexMatchWithLocate<'a, C>);
impl_match_locate!(MultiTextFMIndexMatchWithLocate<'a, C>);
impl_match_text_id!(MultiTextFMIndexMatchWithLocate<'a, C>);
