use num_traits::{Bounded, Num};

/// A character is a type that can be used to store data and to compose a
/// search pattern over this data.
///
/// For instance, when searching in UTF-8 text, characters are u8.
///
/// Characters of types u8, u16, u32 and u64 are supported.
///
/// These can be converted into u64 using `.into()` and from u64 using
/// `from_u64`. When converted from u64, they are truncated.
pub trait Character: Into<u64> + Copy + Clone + Num + Ord + Bounded + std::fmt::Debug {
    /// Take a u64 and convert it into the given data type.
    ///
    /// Truncates the u64 if it is too large to fit in the type.
    fn from_u64(n: u64) -> Self;
}

macro_rules! impl_character {
    ($t:ty) => {
        impl Character for $t {
            fn from_u64(n: u64) -> Self {
                n as Self
            }
        }
    };
}

impl_character!(u64);
impl_character!(u32);
impl_character!(u16);
impl_character!(u8);
