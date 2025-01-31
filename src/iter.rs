use crate::character::Character;
use crate::converter::{Converter, IndexWithConverter};
use crate::search::Search;
use crate::suffix_array::HasPosition;

/// AsCharacters exists so we can have the equivalent of AsRef<[T]> on
/// SearchIndex, but without breaking object-safety. SearchIndex and
/// SearchIndexWithLocate need to be object-safe (dyn-compatible)
pub trait AsCharacters<T: Character> {
    fn as_characters(&self) -> &[T];
}

// Implement for any type that implements AsRef<[T]>
impl<T: Character, A: AsRef<[T]>> AsCharacters<T> for A {
    fn as_characters(&self) -> &[T] {
        self.as_ref()
    }
}

pub trait SearchResult<'a, T: Character> {
    fn search<K: AsRef<[T]>>(&self, pattern: K) -> Self;

    fn count(&self) -> u64;

    fn iter_backward(&self, i: u64) -> impl Iterator<Item = T> + 'a;
    fn iter_forward(&self, i: u64) -> impl Iterator<Item = T> + 'a;
}

pub trait SearchResultWithLocate<'a, T: Character>: SearchResult<'a, T> {
    fn locate(&self) -> Vec<u64>;
}

/// A search index that can be used to search for patterns in a text.
///
/// This only supports the count operation for search, not locate.
pub trait SearchIndex<T: Character> {
    type SearchResult<'a>: SearchResult<'a, T>
    where
        Self: 'a;
    // /// The backend type for this search index.
    // type Backend: FMIndexBackend<T>;

    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    fn search(&self, pattern: &dyn AsCharacters<T>) -> Self::SearchResult<'_>;

    /// The size of the text in the index
    ///
    /// Note that this includes an ending \0 (terminator) character
    /// so will be one more than the length of the text.
    fn len(&self) -> u64;
}

/// A search index that can be used to search for patterns in a text.
///
/// This also supports the locate operation for search.
pub trait SearchIndexWithLocate<T: Character> {
    type SearchResult<'a>: SearchResultWithLocate<'a, T>
    where
        Self: 'a;

    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    fn search(&self, pattern: &dyn AsCharacters<T>) -> Self::SearchResult<'_>;

    /// The size of the text in the index
    ///
    /// Note that this includes an ending \0 (terminator) character
    /// so will be one more than the length of the text.
    fn len(&self) -> u64;
}

/// Trait for an FM-Index implementation.
///
/// You can use this to implement against a FM-Index generically.
///
/// You cannot implement this trait yourself.
pub(crate) trait FMIndexBackend<T: Character> {
    fn get_l(&self, i: u64) -> T;

    fn lf_map(&self, i: u64) -> u64;

    fn lf_map2(&self, c: T, i: u64) -> u64;

    fn get_f(&self, i: u64) -> T;

    fn fl_map(&self, i: u64) -> u64;

    fn fl_map2(&self, c: T, i: u64) -> u64;

    fn len(&self) -> u64;
}

/// Access the heap size of the structure.
///
/// This can be useful if you want to fine-tune the memory usage of your
/// application.
pub trait HeapSize {
    /// The size on the heap of this structure, in bytes.
    fn size(&self) -> usize;
}

/// An iterator that goes backwards through the text, producing [`Character`].
pub struct BackwardIterator<'a, T, I>
where
    T: Character,
    I: FMIndexBackend<T>,
{
    index: &'a I,
    i: u64,
    _t: std::marker::PhantomData<T>,
}

impl<'a, T, I> BackwardIterator<'a, T, I>
where
    T: Character,
    I: FMIndexBackend<T> + IndexWithConverter<T>,
{
    pub(crate) fn new(index: &'a I, i: u64) -> Self {
        BackwardIterator {
            index,
            i,
            _t: std::marker::PhantomData,
        }
    }
}

impl<T, I> Iterator for BackwardIterator<'_, T, I>
where
    T: Character,
    I: FMIndexBackend<T> + IndexWithConverter<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_l(self.i);
        self.i = self.index.lf_map(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}

/// An iterator that goes forwards through the text, producing [`Character`].
pub struct ForwardIterator<'a, T, I>
where
    T: Character,
    I: FMIndexBackend<T>,
{
    index: &'a I,
    i: u64,
    _t: std::marker::PhantomData<T>,
}

impl<'a, T, I> ForwardIterator<'a, T, I>
where
    T: Character,
    I: FMIndexBackend<T>,
{
    pub(crate) fn new(index: &'a I, i: u64) -> Self {
        ForwardIterator {
            index,
            i,
            _t: std::marker::PhantomData,
        }
    }
}

impl<T, I> Iterator for ForwardIterator<'_, T, I>
where
    T: Character,
    I: FMIndexBackend<T> + IndexWithConverter<T>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_f(self.i);
        self.i = self.index.fl_map(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}
