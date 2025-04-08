//! Converters for restricting the alphabet of a [`Character`].
//!

/// If we know a [Character] data type can only consists of particular values,
/// they can be restricted to a smaller alphabet. This helps both speed of
/// search and memory usage.
///
/// A converter can be used to restrict a character of a type to a certain
/// alphabet.
pub trait Converter {
    type Char;

    fn from_u64(&self, c: u64) -> Self::Char;
    fn to_u64(&self, c: Self::Char) -> u64;
    fn from_usize(&self, c: usize) -> Self::Char {
        self.from_u64(c as u64)
    }
    fn to_usize(&self, c: Self::Char) -> usize {
        self.to_u64(c) as usize
    }

    fn max_value(&self) -> Self::Char;
}

pub struct DefaultConverter<T> {
    max_value: T,
}

impl<T> DefaultConverter<T> {
    pub fn new(max_value: T) -> Self {
        DefaultConverter { max_value }
    }
}

macro_rules! impl_default_converter {
    ($t:ty) => {
        impl Default for DefaultConverter<$t> {
            fn default() -> Self {
                DefaultConverter {
                    max_value: <$t>::MAX,
                }
            }
        }

        impl Converter for DefaultConverter<$t> {
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
