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

use crate::character::Character;
use crate::error::Error;
use crate::fm_index::FMIndexBackend;
use crate::multi_pieces::FMIndexMultiPiecesBackend;
use crate::piece::PieceId;
use crate::rlfmi::RLFMIndexBackend;
use crate::suffix_array::discard::DiscardedSuffixArray;
use crate::suffix_array::sample::SOSampledSuffixArray;
use crate::text::Text;
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

    /// The size of the data used by this structure on the heap, in bytes.
    ///
    /// This does not include non-used pre-allocated space.
    fn heap_size(&self) -> usize;
}

/// Trait for searching in an index that supports multiple texts.
pub trait SearchIndexWithMultiPieces<C>: SearchIndex<C> {
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

/// A match in the text that contains the ID of the piece where the pattern is found.
pub trait MatchWithPieceId<'a, C>: Match<'a, C> {
    /// Get the ID of the text that the character at the matched position belongs to.
    fn piece_id(&self) -> PieceId;
}

/// FMIndex, count only.
///
/// The FM-Index is both a search index as well as compact representation of
/// the text.
pub struct FMIndex<C: Character>(SearchIndexWrapper<FMIndexBackend<C, DiscardedSuffixArray>>);
/// Search result for FMIndex, count only.
pub struct FMIndexSearch<'a, C: Character>(
    SearchWrapper<'a, FMIndexBackend<C, DiscardedSuffixArray>>,
);
/// Match in the text for FMIndex.
pub struct FMIndexMatch<'a, C: Character>(
    MatchWrapper<'a, FMIndexBackend<C, DiscardedSuffixArray>>,
);

/// FMIndex with locate support.
///
/// This is an FM-Index which uses additional storage to support locate queries.
pub struct FMIndexWithLocate<C: Character>(
    SearchIndexWrapper<FMIndexBackend<C, SOSampledSuffixArray>>,
);
/// Search result for FMIndex with locate support.
pub struct FMIndexSearchWithLocate<'a, C: Character>(
    SearchWrapper<'a, FMIndexBackend<C, SOSampledSuffixArray>>,
);
/// Match in the text for FMIndex with locate support.
pub struct FMIndexMatchWithLocate<'a, C: Character>(
    MatchWrapper<'a, FMIndexBackend<C, SOSampledSuffixArray>>,
);

/// RLFMIndex, count only.
///
/// This is a version of the FM-Index that uses less space, but is also less efficient.
pub struct RLFMIndex<C: Character>(SearchIndexWrapper<RLFMIndexBackend<C, DiscardedSuffixArray>>);
/// Search result for RLFMIndex, count only.
pub struct RLFMIndexSearch<'a, C: Character>(
    SearchWrapper<'a, RLFMIndexBackend<C, DiscardedSuffixArray>>,
);
/// Match in the text for RLFMIndex.
pub struct RLFMIndexMatch<'a, C: Character>(
    MatchWrapper<'a, RLFMIndexBackend<C, DiscardedSuffixArray>>,
);

/// RLFMIndex with locate support.
///
/// This is a version of the FM-Index that uses less space, but is also less efficient.
/// It uses additional storage to support locate queries.
pub struct RLFMIndexWithLocate<C: Character>(
    SearchIndexWrapper<RLFMIndexBackend<C, SOSampledSuffixArray>>,
);
/// Search result for RLFMIndex with locate support.
pub struct RLFMIndexSearchWithLocate<'a, C: Character>(
    SearchWrapper<'a, RLFMIndexBackend<C, SOSampledSuffixArray>>,
);
/// Match in the text for RLFMIndex with locate support.
pub struct RLFMIndexMatchWithLocate<'a, C: Character>(
    MatchWrapper<'a, RLFMIndexBackend<C, SOSampledSuffixArray>>,
);

/// MultiText index, count only.
///
/// This is a multi-text version of the FM-Index. It allows \0 separated strings.
pub struct FMIndexMultiPieces<C: Character>(
    SearchIndexWrapper<FMIndexMultiPiecesBackend<C, DiscardedSuffixArray>>,
);
/// Search result for MultiText index, count only.
pub struct FMIndexMultiPiecesSearch<'a, C: Character>(
    SearchWrapper<'a, FMIndexMultiPiecesBackend<C, DiscardedSuffixArray>>,
);
/// Match in the text for MultiText index.
pub struct FMIndexMultiPiecesMatch<'a, C: Character>(
    MatchWrapper<'a, FMIndexMultiPiecesBackend<C, DiscardedSuffixArray>>,
);

/// MultiText index with locate support.
///
/// This is a multi-text version of the FM-Index. It allows \0 separated strings.
/// It uses additional storage to support locate queries.
pub struct FMIndexMultiPiecesWithLocate<C: Character>(
    SearchIndexWrapper<FMIndexMultiPiecesBackend<C, SOSampledSuffixArray>>,
);
/// Search result for MultiText index with locate support.
pub struct FMIndexMultiPiecesSearchWithLocate<'a, C: Character>(
    SearchWrapper<'a, FMIndexMultiPiecesBackend<C, SOSampledSuffixArray>>,
);
/// Match in the text for MultiText index with locate support.
pub struct FMIndexMultiPiecesMatchWithLocate<'a, C: Character>(
    MatchWrapper<'a, FMIndexMultiPiecesBackend<C, SOSampledSuffixArray>>,
);

impl<C: Character> FMIndex<C> {
    /// Create a new FMIndex without locate support.
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>) -> Result<Self, Error> {
        Ok(FMIndex(SearchIndexWrapper::new(FMIndexBackend::new(
            text,
            |_| DiscardedSuffixArray {},
        )?)))
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
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>, level: usize) -> Result<Self, Error> {
        Ok(FMIndexWithLocate(SearchIndexWrapper::new(
            FMIndexBackend::new(text, |sa| SOSampledSuffixArray::sample(sa, level))?,
        )))
    }
}

impl<C: Character> RLFMIndex<C> {
    /// Create a new RLFMIndex without locate support.
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>) -> Result<Self, Error> {
        Ok(RLFMIndex(SearchIndexWrapper::new(RLFMIndexBackend::new(
            text,
            |_| DiscardedSuffixArray {},
        )?)))
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
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>, level: usize) -> Result<Self, Error> {
        Ok(RLFMIndexWithLocate(SearchIndexWrapper::new(
            RLFMIndexBackend::new(text, |sa| SOSampledSuffixArray::sample(sa, level))?,
        )))
    }
}

impl<C: Character> FMIndexMultiPieces<C> {
    /// Create a new FMIndexMultiPieces without locate support.
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>) -> Result<Self, Error> {
        Ok(FMIndexMultiPieces(SearchIndexWrapper::new(
            FMIndexMultiPiecesBackend::new(text, |_| DiscardedSuffixArray {})?,
        )))
    }
}

impl<C: Character> FMIndexMultiPiecesWithLocate<C> {
    /// Create a new FMIndexMultiPieces with locate support.
    ///
    /// The level argument controls the sampling rate used. Higher levels use
    /// less storage, at the cost of performance of locate queries. A level of
    /// 0 means no sampling, and a level of 1 means half of the suffix array is
    /// sampled, a level of 2 means a quarter of the suffix array is sampled,
    /// and so on.
    pub fn new<T: AsRef<[C]>>(text: &Text<C, T>, level: usize) -> Result<Self, Error> {
        Ok(FMIndexMultiPiecesWithLocate(SearchIndexWrapper::new(
            FMIndexMultiPiecesBackend::new(text, |sa| SOSampledSuffixArray::sample(sa, level))?,
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

macro_rules! impl_search_index_with_multi_pieces {
    ($t:ty, $s:ident, $st:ty) => {
        impl<C: Character> SearchIndexWithMultiPieces<C> for $t {
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

macro_rules! impl_match_piece_id {
    ($t:ty) => {
        impl<'a, C: Character> MatchWithPieceId<'a, C> for $t {
            fn piece_id(&self) -> PieceId {
                self.0.piece_id()
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
    FMIndexMultiPieces<C>,
    FMIndexMultiPiecesSearch,
    FMIndexMultiPiecesSearch<C>
);
impl_search_index_with_multi_pieces!(
    FMIndexMultiPieces<C>,
    FMIndexMultiPiecesSearch,
    FMIndexMultiPiecesSearch<C>
);
impl_search!(
    FMIndexMultiPiecesSearch<'a, C>,
    FMIndexMultiPiecesMatch,
    FMIndexMultiPiecesMatch<'a, C>
);
impl_match!(FMIndexMultiPiecesMatch<'a, C>);

impl_search_index_with_locate!(
    FMIndexMultiPiecesWithLocate<C>,
    FMIndexMultiPiecesSearchWithLocate,
    FMIndexMultiPiecesSearchWithLocate<C>
);
impl_search_index_with_multi_pieces!(
    FMIndexMultiPiecesWithLocate<C>,
    FMIndexMultiPiecesSearchWithLocate,
    FMIndexMultiPiecesSearchWithLocate<C>
);
impl_search!(
    FMIndexMultiPiecesSearchWithLocate<'a, C>,
    FMIndexMultiPiecesMatchWithLocate,
    FMIndexMultiPiecesMatchWithLocate<'a, C>
);
impl_match!(FMIndexMultiPiecesMatchWithLocate<'a, C>);
impl_match_locate!(FMIndexMultiPiecesMatchWithLocate<'a, C>);
impl_match_piece_id!(FMIndexMultiPiecesMatchWithLocate<'a, C>);
