use crate::Character;

/// A search index.
///
/// Using this trait, you can use [`FMIndex`] and [`RLFMIndex`]
/// interchangeably using generics.
pub trait SearchIndex<T>
where
    T: Character,
{
    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    fn search<K>(&self, pattern: K) -> impl Search<T>
    where
        K: AsRef<[T]>;
}

/// A search index.
///
/// Using this trait, you can use [`FMIndex`] and [`RLFMIndex`]
/// interchangeably using generics.
pub trait SearchIndexWithLocate<T>
where
    T: Character,
{
    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    fn search<K>(&self, pattern: K) -> impl SearchWithLocate<T>
    where
        K: AsRef<[T]>;
}

/// The result of a search.
pub trait Search<T>
where
    T: Character,
{
    /// Search in the current search result, refining it.
    ///
    /// This adds a prefix `pattern` to the existing pattern, and
    /// looks for those expanded patterns in the text.
    fn search<K>(&self, pattern: K) -> Self
    where
        K: AsRef<[T]>;

    /// Get the number of matches.
    fn count(&self) -> u64;

    /// Get an iterator that goes backwards through the text, producing
    /// [`Character`].
    fn iter_backward(&self, i: u64) -> impl Iterator<Item = T>;

    /// Get an iterator that goes forwards through the text, producing
    /// [`Character`].
    fn iter_forward(&self, i: u64) -> impl Iterator<Item = T>;
}

/// The result of a search with a sampled suffix array.
pub trait SearchWithLocate<T>: Search<T>
where
    T: Character,
{
    /// List the position of all occurrences.
    fn locate(&self) -> Vec<u64>;
}
