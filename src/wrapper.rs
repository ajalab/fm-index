// This module provides wrappers around SearchIndexBackend providing
// the functionality used by the frontend.
// This makes the implementation of the frontend more regular.

use num_traits::Zero;

use crate::backend::{HasMultiTexts, HasPosition, SearchIndexBackend};
use crate::converter::Converter;
use crate::text::TextId;
use crate::HeapSize;

pub(crate) struct SearchIndexWrapper<B>(B)
where
    B: SearchIndexBackend;

pub(crate) struct SearchWrapper<'a, B>
where
    B: SearchIndexBackend,
{
    backend: &'a B,
    s: u64,
    e: u64,
    pattern: Vec<B::T>,
    match_prefix_only: bool,
}

impl<B> SearchIndexWrapper<B>
where
    B: SearchIndexBackend + HeapSize,
{
    pub(crate) fn new(backend: B) -> Self {
        SearchIndexWrapper(backend)
    }

    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    pub(crate) fn search<K>(&self, pattern: K) -> SearchWrapper<B>
    where
        K: AsRef<[B::T]>,
    {
        SearchWrapper::new(&self.0, 0, self.0.len(), false).search(pattern)
    }

    /// Get the length of the text in the index.
    ///
    /// Note that this includes an ending \0 (terminator) character
    /// so will be one more than the length of the text passed in.
    pub(crate) fn len(&self) -> u64 {
        self.0.len()
    }

    pub(crate) fn heap_size(&self) -> usize {
        B::heap_size(&self.0)
    }
}

impl<B> SearchIndexWrapper<B>
where
    B: SearchIndexBackend + HasMultiTexts,
{
    pub(crate) fn search_prefix<K>(&self, pattern: K) -> SearchWrapper<B>
    where
        K: AsRef<[B::T]>,
    {
        SearchWrapper::new(&self.0, 0, self.0.len(), true).search(pattern)
    }

    /// Search for the text which has the given suffix.
    pub(crate) fn search_suffix<K>(&self, pattern: K) -> SearchWrapper<B>
    where
        K: AsRef<[B::T]>,
    {
        SearchWrapper::new(&self.0, 0, self.0.text_count(), false).search(pattern)
    }

    /// Search for a pattern that is an exact match of a text.
    pub(crate) fn search_exact<K>(&self, pattern: K) -> SearchWrapper<B>
    where
        K: AsRef<[B::T]>,
    {
        SearchWrapper::new(&self.0, 0, self.0.text_count(), true).search(pattern)
    }
}

impl<'a, B> SearchWrapper<'a, B>
where
    B: SearchIndexBackend,
{
    fn new(backend: &'a B, s: u64, e: u64, match_prefix_only: bool) -> Self {
        SearchWrapper {
            backend,
            s,
            e,
            pattern: vec![],
            match_prefix_only,
        }
    }

    /// Search in the current search result, refining it.
    ///
    /// This adds a prefix `pattern` to the existing pattern, and
    /// looks for those expanded patterns in the text.
    pub(crate) fn search<K: AsRef<[B::T]>>(&self, pattern: K) -> Self {
        // TODO: move this loop into backend to avoid dispatch overhead
        let mut s = self.s;
        let mut e = self.e;
        let mut pattern = pattern.as_ref().to_vec();
        for &c in pattern.iter().rev() {
            s = self.backend.lf_map2(c, s);
            e = self.backend.lf_map2(c, e);
            if s == e {
                break;
            }
        }
        pattern.extend_from_slice(&self.pattern);

        SearchWrapper {
            backend: self.backend,
            s,
            e,
            pattern,
            match_prefix_only: self.match_prefix_only,
        }
    }

    #[cfg(test)]
    pub(crate) fn get_range(&self) -> (u64, u64) {
        (self.s, self.e)
    }

    /// Count the number of occurrences.
    pub(crate) fn count(&self) -> u64 {
        self.e - self.s
    }

    // Iterate all occurrences of the found patterns.
    pub(crate) fn iter_matches(&self) -> impl Iterator<Item = MatchWrapper<'a, B>> {
        MatchIteratorWrapper::new(self.backend, self.s, self.e, self.match_prefix_only)
    }
}

/// An iterator that goes backwards through the text, producing [`Character`].
pub(crate) struct BackwardIteratorWrapper<'a, B: SearchIndexBackend> {
    backend: &'a B,
    i: u64,
}

impl<'a, B: SearchIndexBackend> BackwardIteratorWrapper<'a, B> {
    pub(crate) fn new(backend: &'a B, i: u64) -> Self {
        BackwardIteratorWrapper { backend, i }
    }
}

impl<B: SearchIndexBackend> Iterator for BackwardIteratorWrapper<'_, B> {
    type Item = B::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.backend.get_l(self.i);
        self.i = self.backend.lf_map(self.i);
        Some(self.backend.get_converter().convert_inv(c))
    }
}

/// An iterator that goes forwards through the text, producing [`Character`].
pub(crate) struct ForwardIteratorWrapper<'a, B: SearchIndexBackend> {
    backend: &'a B,
    i: u64,
}

impl<'a, B: SearchIndexBackend> ForwardIteratorWrapper<'a, B> {
    pub(crate) fn new(backend: &'a B, i: u64) -> Self {
        ForwardIteratorWrapper { backend, i }
    }
}

impl<B: SearchIndexBackend> Iterator for ForwardIteratorWrapper<'_, B> {
    type Item = B::T;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.backend.get_f(self.i);
        self.i = self.backend.fl_map(self.i)?;
        Some(self.backend.get_converter().convert_inv(c))
    }
}

pub(crate) struct MatchIteratorWrapper<'a, B: SearchIndexBackend> {
    backend: &'a B,
    i: u64,
    e: u64,
    match_prefix_only: bool,
}

impl<'a, B: SearchIndexBackend> MatchIteratorWrapper<'a, B> {
    pub(crate) fn new(backend: &'a B, i: u64, e: u64, match_prefix_only: bool) -> Self {
        MatchIteratorWrapper {
            backend,
            i,
            e,
            match_prefix_only,
        }
    }
}

impl<'a, B: SearchIndexBackend> Iterator for MatchIteratorWrapper<'a, B> {
    type Item = MatchWrapper<'a, B>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.e {
            if !self.match_prefix_only || self.backend.get_l(self.i).is_zero() {
                let location = MatchWrapper::new(self.backend, self.i);
                self.i += 1;
                return Some(location);
            }
            self.i += 1;
        }
        None
    }
}

pub(crate) struct MatchWrapper<'a, B: SearchIndexBackend> {
    backend: &'a B,
    i: u64,
}

impl<'a, B: SearchIndexBackend> MatchWrapper<'a, B> {
    pub(crate) fn new(backend: &'a B, i: u64) -> Self {
        MatchWrapper { backend, i }
    }

    pub(crate) fn iter_chars_forward(&self) -> impl Iterator<Item = B::T> + use<'a, B> {
        ForwardIteratorWrapper::new(self.backend, self.i)
    }

    pub(crate) fn iter_chars_backward(&self) -> impl Iterator<Item = B::T> + use<'a, B> {
        BackwardIteratorWrapper::new(self.backend, self.i)
    }
}

impl<B: SearchIndexBackend + HasPosition> MatchWrapper<'_, B> {
    pub(crate) fn locate(&self) -> u64 {
        self.backend.get_sa(self.i)
    }
}

impl<B: SearchIndexBackend + HasMultiTexts> MatchWrapper<'_, B> {
    pub(crate) fn text_id(&self) -> TextId {
        self.backend.text_id(self.i)
    }
}
