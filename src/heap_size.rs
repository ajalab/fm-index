/// Access the heap size of the structure.
///
/// This can be useful if you want to fine-tune the memory usage of your
/// application.
pub(crate) trait HeapSize {
    /// The size of the data used by this structure on the heap, in bytes.
    ///
    /// This does not include non-used pre-allocated space.
    fn heap_size(&self) -> usize;
}
