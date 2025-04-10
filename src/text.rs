use crate::util;
use crate::Character;
use num_traits::Bounded;

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

/// A text structure used as the target for pattern searching in the index.
///
/// Not only does it contain the text, but also the maximum character value in the
/// text. This is used to determine the number of bits needed to store the
/// characters in the text.
pub struct Text<C, T>
where
    C: Character,
    T: AsRef<[C]>,
{
    text: T,
    max_character: C,
}

impl<C, T> Text<C, T>
where
    C: Character + Bounded,
    T: AsRef<[C]>,
{
    /// Create a new text structure with the given text.
    ///
    /// The maximum character value is set to the maximum value of the
    /// character type.
    pub fn new(text: T) -> Self {
        Text {
            text,
            max_character: C::max_value(),
        }
    }
}

impl<C, T> Text<C, T>
where
    C: Character,
    T: AsRef<[C]>,
{
    /// Create a new text structure with the given text and the largest character value.
    ///
    /// This is used when the maximum character value is known in advance.
    pub fn with_max_character(text: T, max_character: C) -> Self {
        Text {
            text,
            max_character,
        }
    }

    /// Get the text as a slice.
    pub fn text(&self) -> &[C] {
        self.text.as_ref()
    }

    /// Return the maximum character value in the text.
    pub fn max_character(&self) -> C {
        self.max_character
    }

    pub(crate) fn max_bits(&self) -> usize {
        util::log2_usize(self.max_character().into_usize()) + 1
    }
}
