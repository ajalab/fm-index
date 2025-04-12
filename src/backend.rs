use crate::character::Character;
use crate::doc::DocId;

/// Trait for an FM-Index backend implementation
pub(crate) trait SearchIndexBackend: Sized {
    /// A [`Character`] type.
    type C: Character;

    // We hide all the methods involved in implementation.

    fn get_l(&self, i: usize) -> Self::C;

    fn lf_map(&self, i: usize) -> usize;

    fn lf_map2(&self, c: Self::C, i: usize) -> usize;

    fn get_f(&self, i: usize) -> Self::C;

    fn fl_map(&self, i: usize) -> Option<usize>;

    /// The size of the text in the index
    ///
    /// Note that this includes an ending \0 (terminator) character
    /// so will be one more than the length of the text.
    fn len(&self) -> usize;
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

/// A trait for an index that contains multiple documents.
pub(crate) trait HasMultiDocs {
    /// Returns the ID of the document that the character at the given position on the suffix array belongs to.
    fn doc_id(&self, i: usize) -> DocId;

    /// Returns the number of documents in the index.
    fn docs_count(&self) -> usize;
}
