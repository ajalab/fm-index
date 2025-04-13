use crate::heap_size::HeapSize;

pub struct DiscardedSuffixArray {}

impl HeapSize for DiscardedSuffixArray {
    fn heap_size(&self) -> usize {
        0
    }
}
