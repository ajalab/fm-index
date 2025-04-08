/// A unique id identifying this text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct TextId(usize);

impl From<usize> for TextId {
    fn from(value: usize) -> Self {
        TextId(value)
    }
}

impl From<TextId> for usize {
    fn from(value: TextId) -> usize {
        value.0
    }
}
