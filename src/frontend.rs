use crate::{converter::Converter, Character};

/// A search index.
///
/// Using this trait, you can use [`FMIndex`] and [`RLFMIndex`]
/// interchangeably using generics.
pub trait SearchIndex<T, C>
where
    T: Character,
    C: Converter<T>,
{
    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    fn search<K>(&self, pattern: K) -> impl Search<T, C>
    where
        K: AsRef<[T]>;
}

pub trait Search<T, C>
where
    T: Character,
    C: Converter<T>,
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
}

pub trait SearchWithLocate<T, C>: Search<T, C>
where
    T: Character,
    C: Converter<T>,
{
    /// List the position of all occurrences.
    fn locate(&self) -> Vec<u64>;
}
