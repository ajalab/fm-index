use crate::converter::Converter;
use crate::text::TextId;

/// Trait for an FM-Index backend implementation
pub(crate) trait SearchIndexBackend: Sized {
    type T: Copy + Clone;
    type C: Converter<Char = Self::T>;

    // We hide all the methods involved in implementation.

    fn get_l(&self, i: usize) -> Self::T;

    fn lf_map(&self, i: usize) -> usize;

    fn lf_map2(&self, c: Self::T, i: usize) -> usize;

    fn get_f(&self, i: usize) -> Self::T;

    fn fl_map(&self, i: usize) -> Option<usize>;

    /// The size of the text in the index
    ///
    /// Note that this includes an ending \0 (terminator) character
    /// so will be one more than the length of the text.
    fn len(&self) -> usize;

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
    fn get_sa(&self, i: usize) -> usize;
}

/// A trait for an index that contains multiple texts.
pub(crate) trait HasMultiTexts {
    /// Returns the ID of the text that the character at the given position on the suffix array belongs to.
    fn text_id(&self, i: usize) -> TextId;

    /// Returns the number of texts in the index.
    fn text_count(&self) -> usize;
}
