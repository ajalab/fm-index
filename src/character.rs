use crate::util;

/// A character is a type that can be used to store data and to compose a
/// search pattern over this data.
///
/// For instance, when searching in UTF-8 text, characters are u8.
///
/// Characters of types u8, u16, u32 and u64 are supported.
///
/// These can be converted into u64 using `.into()` and from u64 using
/// `from_u64`. When converted from u64, they are truncated.
pub trait Character: Copy + Clone {
    fn into_u64(self) -> u64;

    fn from_u64(x: u64) -> Self;

    fn into_usize(self) -> usize {
        self.into_u64() as usize
    }

    fn from_usize(x: usize) -> Self {
        Self::from_u64(x as u64)
    }
}

macro_rules! impl_character {
    ($t:ty) => {
        impl Character for $t {
            fn into_u64(self) -> u64 {
                self as u64
            }

            fn from_u64(x: u64) -> Self {
                x as $t
            }
        }
    };
}

impl_character!(u64);
impl_character!(u32);
impl_character!(u16);
impl_character!(u8);
impl_character!(usize);
