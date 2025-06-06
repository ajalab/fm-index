use crate::character::Character;
use crate::piece::PieceId;

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

/// A trait for an index that supports locate queries.
pub(crate) trait HasPosition {
    fn get_sa(&self, i: usize) -> usize;
}

/// A trait for an index that contains multiple pieces (text fragments).
pub(crate) trait HasMultiPieces {
    /// Returns the ID of the piece the character at the given position on the suffix array belongs to.
    fn piece_id(&self, i: usize) -> PieceId;

    /// Returns the number of pieces in the index.
    fn pieces_count(&self) -> usize;
}
