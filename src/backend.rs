use crate::character::Character;
use crate::converter::Converter;

/// Trait for an FM-Index backend implementation
pub(crate) trait SearchIndexBackend: Sized {
    /// A [`Character`] type.
    type T: Character;
    type C: Converter<Self::T>;

    // We hide all the methods involved in implementation.

    fn get_l(&self, i: u64) -> Self::T;

    fn lf_map(&self, i: u64) -> u64;

    fn lf_map2(&self, c: Self::T, i: u64) -> u64;

    fn get_f(&self, i: u64) -> Self::T;

    fn fl_map(&self, i: u64) -> u64;

    fn fl_map2(&self, c: Self::T, i: u64) -> u64;

    /// The size of the text in the index
    ///
    /// Note that this includes an ending \0 (terminator) character
    /// so will be one more than the length of the text.
    fn len(&self) -> u64;

    /// Get the converter for this index.
    fn get_converter(&self) -> &Self::C;
}

/// Access the heap size of the structure.
///
/// This can be useful if you want to fine-tune the memory usage of your
/// application.
pub trait HeapSize {
    /// The size on the heap of this structure, in bytes.
    fn heap_size(&self) -> usize;
}

/// A trait for an index that supports locate queries.
pub(crate) trait HasPosition {
    fn get_sa(&self, i: u64) -> u64;
}
