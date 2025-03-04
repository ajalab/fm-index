/// A unique id identifying this text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct TextId(u64);

impl From<u64> for TextId {
    fn from(value: u64) -> Self {
        TextId(value)
    }
}

impl From<TextId> for u64 {
    fn from(value: TextId) -> u64 {
        value.0
    }
}
