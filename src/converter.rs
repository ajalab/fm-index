//! Converters for converting into numerical representation.
//!

/// If we know a [Character] data type can only consists of particular values,
/// they can be restricted to a smaller alphabet. This helps both speed of
/// search and memory usage.
///
/// A converter can be used to restrict a character of a type to a certain
/// alphabet.
pub trait Converter {
    /// The character type used by this converter.
    type Char;

    /// Convert a u64 into a character of this type.
    #[allow(clippy::wrong_self_convention)]
    fn from_u64(&self, c: u64) -> Self::Char;

    /// Convert a character of this type into a u64.
    fn to_u64(&self, c: Self::Char) -> u64;

    /// Convert a usize into a character of this type.
    #[allow(clippy::wrong_self_convention)]
    fn from_usize(&self, c: usize) -> Self::Char {
        self.from_u64(c as u64)
    }

    /// Convert a character of this type into a usize.
    fn to_usize(&self, c: Self::Char) -> usize {
        self.to_u64(c) as usize
    }

    /// Returns the maximum value of this character type.
    fn max_value(&self) -> Self::Char;
}

/// A no-op converter that does not change the character representation.
pub struct NoOpConverter<T> {
    max_value: T,
}

impl<T> NoOpConverter<T> {
    /// Creates a new `NoOpConverter` with the given maximum value.
    pub fn new(max_value: T) -> Self {
        NoOpConverter { max_value }
    }
}

macro_rules! impl_default_converter {
    ($t:ty) => {
        impl Default for NoOpConverter<$t> {
            fn default() -> Self {
                NoOpConverter {
                    max_value: <$t>::MAX,
                }
            }
        }

        impl Converter for NoOpConverter<$t> {
            type Char = $t;
            fn from_u64(&self, c: u64) -> $t {
                c as $t
            }
            fn to_u64(&self, c: $t) -> u64 {
                c as u64
            }
            fn max_value(&self) -> $t {
                self.max_value
            }
        }
    };
}

impl_default_converter!(u64);
impl_default_converter!(u32);
impl_default_converter!(u16);
impl_default_converter!(u8);
impl_default_converter!(usize);
