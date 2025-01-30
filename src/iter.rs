use crate::character::Character;
use crate::converter::{Converter, IndexWithConverter};
use crate::seal;
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

/// A search index that can be used to search for patterns in a text.
///
/// This only supports the count operation for search, not locate.
pub trait SearchIndex<T: Character> {
    /// The backend type for this search index.
    type Backend: FMIndexBackend<T = T>;

    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    fn search(&self, pattern: &dyn AsCharacters<T>) -> Search<Self::Backend>;

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
    /// The backend type for this search index.
    type Backend: FMIndexBackend<T = T> + HasPosition;

    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    fn search(&self, pattern: &dyn AsCharacters<T>) -> Search<Self::Backend>;

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
pub trait FMIndexBackend: Sized + seal::Sealed {
    /// A [`Character`] type.
    type T: Character;

    // We hide all the methods involved in implementation.

    #[doc(hidden)]
    fn get_l<L: seal::IsLocal>(&self, i: u64) -> Self::T;
    #[doc(hidden)]
    fn lf_map<L: seal::IsLocal>(&self, i: u64) -> u64;
    #[doc(hidden)]
    fn lf_map2<L: seal::IsLocal>(&self, c: Self::T, i: u64) -> u64;
    #[doc(hidden)]
    fn get_f<L: seal::IsLocal>(&self, i: u64) -> Self::T;
    #[doc(hidden)]
    fn fl_map<L: seal::IsLocal>(&self, i: u64) -> u64;
    #[doc(hidden)]
    fn fl_map2<L: seal::IsLocal>(&self, c: Self::T, i: u64) -> u64;

    #[doc(hidden)]
    fn iter_forward<L: seal::IsLocal>(&self, i: u64) -> ForwardIterator<Self> {
        debug_assert!(i < self.len::<L>());
        ForwardIterator { index: self, i }
    }

    #[doc(hidden)]
    fn iter_backward<L: seal::IsLocal>(&self, i: u64) -> BackwardIterator<Self> {
        debug_assert!(i < self.len::<L>());
        BackwardIterator { index: self, i }
    }

    #[doc(hidden)]
    fn len<L: seal::IsLocal>(&self) -> u64;
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
pub struct BackwardIterator<'a, I>
where
    I: FMIndexBackend,
{
    index: &'a I,
    i: u64,
}

impl<T, I> Iterator for BackwardIterator<'_, I>
where
    T: Character,
    I: FMIndexBackend<T = T> + IndexWithConverter<T>,
{
    type Item = <I as FMIndexBackend>::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_l::<seal::Local>(self.i);
        self.i = self.index.lf_map::<seal::Local>(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}

/// An iterator that goes forwards through the text, producing [`Character`].
pub struct ForwardIterator<'a, I>
where
    I: FMIndexBackend,
{
    index: &'a I,
    i: u64,
}

impl<T, I> Iterator for ForwardIterator<'_, I>
where
    T: Character,
    I: FMIndexBackend<T = T> + IndexWithConverter<T>,
{
    type Item = <I as FMIndexBackend>::T;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.index.get_f::<seal::Local>(self.i);
        self.i = self.index.fl_map::<seal::Local>(self.i);
        Some(self.index.get_converter().convert_inv(c))
    }
}
