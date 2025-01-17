use crate::iter::{BackwardIterableIndex, BackwardIterator, ForwardIterableIndex, ForwardIterator};
use crate::seal;
use crate::suffix_array::HasPosition;

#[cfg(doc)]
use crate::character::Character;
#[cfg(doc)]
use crate::fm_index::FMIndex;
#[cfg(doc)]
use crate::rlfmi::RLFMIndex;
use crate::text_builder::TextId;

/// A search index.
///
/// Using this trait, you can use [`FMIndex`] and [`RLFMIndex`]
/// interchangeably using generics.
pub trait SearchIndex: BackwardIterableIndex {
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

    // If we created a HasDoc trait (or something better named) for those
    // indexes that maintain the `Doc` structure, we could move the following
    // methods into a trait that depends on that.

    /// Given a text id, return the text associated with it.
    ///
    /// This is the actual text, excluding zero separators.
    fn text(&self, id: TextId) -> &[Self::T] {
        todo!()
    }

    /// Search for texts that contain pattern.
    ///
    ///
    /// This is identical to search(), except if pattern were to
    /// contain a null character. (should we allow it?)
    fn search_contains<K>(&self, pattern: K) -> Search<Self>
    where
        K: AsRef<[Self::T]>,
    {
        todo!();
    }

    /// Search for texts that start with pattern.
    fn search_start_with<K>(&self, pattern: K) -> Search<Self>
    where
        K: AsRef<[Self::T]>,
    {
        todo!();
    }

    /// Search for texts that end with pattern.
    fn search_ends_with<K>(&self, pattern: K) -> Search<Self>
    where
        K: AsRef<[Self::T]>,
    {
        todo!();
    }

    /// Search for texts that are exactly pattern.
    fn search_exact<K>(&self, pattern: K) -> Search<Self>
    where
        K: AsRef<[Self::T]>,
    {
        todo!();
    }
}

impl<I: BackwardIterableIndex> SearchIndex for I {}

/// An object containing the result of a search.
///
/// This is expanded with a `locate` method if the index is
/// supplied with a sampled suffix array.
pub struct Search<'a, I>
where
    I: SearchIndex,
{
    index: &'a I,
    s: u64,
    e: u64,
    pattern: Vec<I::T>,
}

impl<'a, I> Search<'a, I>
where
    I: SearchIndex,
{
    fn new(index: &'a I) -> Search<'a, I> {
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
    I: BackwardIterableIndex,
{
    /// Get an iterator that goes backwards through the text, producing
    /// [`Character`].
    pub fn iter_backward(&self, i: u64) -> BackwardIterator<I> {
        let m = self.count();

        debug_assert!(m > 0, "cannot iterate from empty search result");
        debug_assert!(i < m, "{} is out of range", i);

        self.index.iter_backward::<seal::Local>(self.s + i)
    }
}

impl<I> Search<'_, I>
where
    I: SearchIndex + ForwardIterableIndex,
{
    /// Get an iterator that goes forwards through the text, producing
    /// [`Character`].
    pub fn iter_forward(&self, i: u64) -> ForwardIterator<I> {
        let m = self.count();

        debug_assert!(m > 0, "cannot iterate from empty search result");
        debug_assert!(i < m, "{} is out of range", i);

        self.index.iter_forward::<seal::Local>(self.s + i)
    }
}

impl<I> Search<'_, I>
where
    I: SearchIndex + HasPosition,
{
    /// List the position of all occurrences.
    pub fn locate(&self) -> Vec<u64> {
        let mut results: Vec<u64> = Vec::with_capacity((self.e - self.s) as usize);
        for k in self.s..self.e {
            results.push(self.index.get_sa::<seal::Local>(k));
        }
        results
    }

    /// List the position of all occurrences with an iterator.
    ///
    /// TODO: we could also provide an `IntoIterator` for seach that returns this.
    pub fn locate_iter(&self) -> LocationInfoIterator<I> {
        LocationInfoIterator::new(self.index, self.s, self.e)
    }
}

pub struct LocationInfoIterator<'a, I>
where
    I: SearchIndex + HasPosition,
{
    index: &'a I,
    k_iterator: std::ops::Range<u64>,
}

impl<'a, I> LocationInfoIterator<'a, I>
where
    I: SearchIndex + HasPosition,
{
    pub fn new(index: &'a I, start: u64, end: u64) -> Self {
        LocationInfoIterator {
            index,
            k_iterator: start..end,
        }
    }
}

impl<'a, I> Iterator for LocationInfoIterator<'a, I>
where
    I: SearchIndex + HasPosition,
{
    type Item = LocationInfo<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        let k = self.k_iterator.next()?;
        Some(LocationInfo {
            index: self.index,
            k,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.k_iterator.size_hint()
    }
}

pub struct LocationInfo<'a, I>
where
    I: SearchIndex + HasPosition,
{
    index: &'a I,
    k: u64,
}

impl<I> LocationInfo<'_, I>
where
    I: SearchIndex + HasPosition,
{
    /// the position of a location within the larger text
    pub fn position(&self) -> u64 {
        self.index.get_sa::<seal::Local>(self.k)
    }

    // the existence of the following methods could depend on the
    // `HasDoc` trait.

    /// the text id that the location belongs to.
    ///
    /// Each 0 separated text has a unique id identifying it.
    pub fn text_id(&self) -> TextId {
        todo!()
    }

    /// the original text at this text id
    ///
    /// This does not include the 0 characters at its boundaries.
    pub fn text(&self) -> &[I::T] {
        todo!()
    }
}
