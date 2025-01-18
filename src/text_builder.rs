use crate::Character;

/// A text builder lets you construct a text from multiple parts.
/// Internally each part is separated by the 0 character.
pub struct TextBuilder<T>
where
    T: Character,
{
    id_counter: usize,
    text: Vec<T>,
}

/// A unique id identifying this text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct TextId(usize);

impl<T> TextBuilder<T>
where
    T: Character,
{
    /// Create a new empty text builder.
    pub fn new() -> TextBuilder<T> {
        TextBuilder {
            id_counter: 0,
            text: vec![],
        }
    }

    /// Add a text to the builder.
    ///
    /// Returns a unique id for this text.
    fn add_text(&mut self, text: &[T]) -> TextId {
        let id = TextId(self.id_counter);
        self.id_counter += 1;
        self.text.extend_from_slice(text);
        self.text.push(T::zero());
        id
    }

    /// Finish the build and return the text.
    fn build(self) -> Vec<T> {
        self.text
    }
}
