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
    pub fn with_max_character(text: T, max_character: C) -> Self {
        Text {
            text,
            max_character,
        }
    }

    pub fn text(&self) -> &[C] {
        self.text.as_ref()
    }

    pub fn max_character(&self) -> C {
        self.max_character
    }

    pub(crate) fn max_bits(&self) -> usize {
        util::log2_usize(self.max_character().into_usize()) + 1
    }
}
