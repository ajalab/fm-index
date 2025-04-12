/// A unique id identifying a single document in a text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct DocId(usize);

impl From<usize> for DocId {
    fn from(value: usize) -> Self {
        DocId(value)
    }
}

impl From<DocId> for usize {
    fn from(value: DocId) -> usize {
        value.0
    }
}
