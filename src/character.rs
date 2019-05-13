use num_traits::Num;

pub trait Character: Into<u64> + Copy + Clone + Num + Ord + std::fmt::Debug {
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
