#[cfg(doc)]
use crate::character::Character;
#[cfg(doc)]
use crate::converter;

use crate::converter::IndexWithConverter;
use crate::iter::FMIndexBackend;
use crate::seal;
use crate::suffix_array::HasPosition;

/// A full-text index backed by FM-Index or its variant.
pub struct SearchIndex<I: FMIndexBackend> {
    index: I,
}

impl<I: FMIndexBackend> SearchIndex<I> {
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
pub struct Search<'a, I: FMIndexBackend> {
    index: &'a I,
    s: u64,
    e: u64,
    pattern: Vec<I::T>,
}

impl<'a, I> Search<'a, I>
where
    I: FMIndexBackend,
{
    pub(crate) fn new(index: &'a I) -> Search<'a, I> {
        Search {
            index,
            s: 0,
            e: index.len(),
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
    I: FMIndexBackend + IndexWithConverter<I::T>,
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
    I: FMIndexBackend + IndexWithConverter<I::T>,
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
    I: FMIndexBackend + HasPosition,
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
