use crate::{converter::Converter, FMIndexBackend, HasPosition};

struct SearchIndexWrapper<B>(B)
where
    B: FMIndexBackend;

struct SearchIndexWithLocateWrapper<B>(B)
where
    B: FMIndexBackend + HasPosition;

struct SearchWrapper<'a, B>
where
    B: FMIndexBackend,
{
    backend: &'a B,
    s: u64,
    e: u64,
    pattern: Vec<B::T>,
}

// struct SearchWithLocateWrapper<'a, B>(&'a B)
// where
//     B: FMIndexBackend + HasPosition;

impl<B> SearchIndexWrapper<B>
where
    B: FMIndexBackend,
{
    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    pub fn search<K>(&self, pattern: K) -> SearchWrapper<B>
    where
        K: AsRef<[B::T]>,
    {
        SearchWrapper::new(&self.0).search(pattern)
    }

    /// Get the length of the text in the index.
    ///
    /// Note that this includes an ending \0 (terminator) character
    /// so will be one more than the length of the text passed in.
    pub fn len(&self) -> u64 {
        self.0.len()
    }
}

impl<'a, B> SearchWrapper<'a, B>
where
    B: FMIndexBackend,
{
    pub(crate) fn new(backend: &'a B) -> Self {
        let e = backend.len();
        SearchWrapper {
            backend,
            s: 0,
            e,
            pattern: vec![],
        }
    }

    /// Search in the current search result, refining it.
    ///
    /// This adds a prefix `pattern` to the existing pattern, and
    /// looks for those expanded patterns in the text.
    pub fn search<K: AsRef<[B::T]>>(&self, pattern: K) -> Self {
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
        }
    }

    /// Count the number of occurrences.
    pub fn count(&self) -> u64 {
        self.e - self.s
    }

    /// Get an iterator that goes backwards through the text, producing
    /// [`Character`].
    pub fn iter_backward(&self, i: u64) -> impl Iterator<Item = B::T> + use<'a, B> {
        let m = self.count();

        debug_assert!(m > 0, "cannot iterate from empty search result");
        debug_assert!(i < m, "{} is out of range", i);

        debug_assert!(i < self.backend.len());
        BackwardIterator::new(self.backend, self.s + i)
    }

    // Get an iterator that goes forwards through the text, producing
    /// [`Character`].
    pub fn iter_forward(&self, i: u64) -> impl Iterator<Item = B::T> + use<'a, B> {
        let m = self.count();

        debug_assert!(m > 0, "cannot iterate from empty search result");
        debug_assert!(i < m, "{} is out of range", i);
        debug_assert!(i < self.backend.len());

        ForwardIterator::new(self.backend, self.s + i)
    }
}

/// An iterator that goes backwards through the text, producing [`Character`].
pub struct BackwardIterator<'a, B: FMIndexBackend> {
    backend: &'a B,
    i: u64,
}

impl<'a, B: FMIndexBackend> BackwardIterator<'a, B> {
    pub(crate) fn new(backend: &'a B, i: u64) -> Self {
        BackwardIterator { backend, i }
    }
}

impl<'a, B: FMIndexBackend> Iterator for BackwardIterator<'a, B> {
    type Item = B::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.backend.get_l(self.i);
        self.i = self.backend.lf_map(self.i);
        Some(self.backend.get_converter().convert_inv(c))
    }
}

/// An iterator that goes forwards through the text, producing [`Character`].
pub struct ForwardIterator<'a, B: FMIndexBackend> {
    backend: &'a B,
    i: u64,
}

impl<'a, B: FMIndexBackend> ForwardIterator<'a, B> {
    pub(crate) fn new(backend: &'a B, i: u64) -> Self {
        ForwardIterator { backend, i }
    }
}

impl<'a, B: FMIndexBackend> Iterator for ForwardIterator<'a, B> {
    type Item = B::T;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.backend.get_f(self.i);
        self.i = self.backend.fl_map(self.i);
        Some(self.backend.get_converter().convert_inv(c))
    }
}

// pub struct FMIndex<T: Character, C: Converter<T>>(FMIndexBackendImpl<T, C, ()>);

// pub struct FMIndexWithLocate<T: Character, C: Converter<T>>(
//     FMIndexBackendImpl<T, C, SuffixOrderSampledArray>,
// );

// // traits:
// // search index
// // search
// // search with locate
