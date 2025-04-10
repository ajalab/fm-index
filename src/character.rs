/// A character is a type that can be used to store data and to compose a
/// search pattern over this data.
///
/// These can be converted into u64 using `.into()` and from u64 using
/// `from_u64`. When converted from u64, they are truncated.
pub trait Character: Copy + Clone {
    /// Convert the character into a u64.
    fn into_u64(self) -> u64;

    /// Convert a u64 into a character.
    fn from_u64(x: u64) -> Self;

    /// Convert the character into a u32.
    fn into_usize(self) -> usize {
        self.into_u64() as usize
    }

    /// Convert a u32 into a character.
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
