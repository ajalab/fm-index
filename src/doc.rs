/// A unique id identifying a single document in a text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct PieceId(usize);

impl From<usize> for PieceId {
    fn from(value: usize) -> Self {
        PieceId(value)
    }
}

impl From<PieceId> for usize {
    fn from(value: PieceId) -> usize {
        value.0
    }
}
