//! Converters for restricting the alphabet of a [`Character`].
//!

use crate::character::Character;

use serde::{Deserialize, Serialize};

/// If we know a [Character] data type can only consists of particular values,
/// they can be restricted to a smaller alphabet. This helps both speed of
/// search and memory usage.
///
/// A converter can be used to restrict a character of a type to a certain
/// alphabet.
pub trait Converter<T>
where
    T: Character,
{
    /// Convert a character to a new character in the restricted alphabet.
    fn convert(&self, c: T) -> T;
    /// Convert a character back to the original character.
    fn convert_inv(&self, c: T) -> T;
    /// Get the size of the restricted alphabet.
    fn len(&self) -> u64;
}

/// Restrict characters to a range defining the alphabet.
///
/// Characters are normalized to fit in the range `1..=(max - min)`.
///
/// The range is defined by the minimum and maximum values of the alphabet.
///
/// The null (zero) character is handled separately and is always accepted.
#[derive(Serialize, Deserialize)]
pub struct RangeConverter<T>
where
    T: Character,
{
    min: T,
    max: T,
}

impl<T> RangeConverter<T>
where
    T: Character,
{
    /// Create a new converter that restricts characters to a range.
    pub fn new(min: T, max: T) -> Self {
        debug_assert!(!T::is_zero(&min), "min should not be zero");
        RangeConverter { min, max }
    }
}

impl<T> Converter<T> for RangeConverter<T>
where
    T: Character,
{
    fn convert(&self, c: T) -> T {
        if c == T::zero() {
            c
        } else {
            c - self.min + T::one()
        }
    }

    fn convert_inv(&self, c: T) -> T {
        if c == T::zero() {
            c
        } else {
            c + self.min - T::one()
        }
    }

    fn len(&self) -> u64 {
        // [min, max] + sentinel
        (self.max - self.min).into() + 2
    }
}

/// An identity converter that does not restrict the alphabet.
pub struct IdConverter {
    size: u64,
}

impl IdConverter {
    /// Construct a new IdConverter for a given character type.
    ///
    /// Example:
    ///
    /// ```
    /// use fm_index::converter::IdConverter;
    ///
    /// IdConverter::new::<u8>();
    /// ```
    pub fn new<T: Character>() -> Self {
        IdConverter {
            size: T::max_value().into() + 1,
        }
    }

    /// Create a new IdConverter.
    ///
    /// The size given should be the size of the alphabet, so for u8 it would
    /// be 256, for u16 it would be 65536, etc
    pub(crate) fn with_size(size: u64) -> Self {
        IdConverter { size }
    }
}

impl<T> Converter<T> for IdConverter
where
    T: Character,
{
    fn convert(&self, c: T) -> T {
        c
    }
    fn convert_inv(&self, c: T) -> T {
        c
    }
    fn len(&self) -> u64 {
        self.size
    }
}
