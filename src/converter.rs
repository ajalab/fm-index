use crate::util;

use num_traits::{self, Num};

pub trait Converter<T> {
    fn convert(&self, c: T) -> T;
    fn convert_inv(&self, c: T) -> T;
    fn size(&self) -> u64;
    fn len(&self) -> u64;
}

pub struct RangeConverter<T> {
    min: T,
    max: T,
}

impl<T> RangeConverter<T> {
    pub fn new(min: T, max: T) -> Self {
        return RangeConverter { min: min, max: max };
    }
}

impl<T> Converter<T> for RangeConverter<T>
where
    T: Copy + Clone + Into<u64> + Num,
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
        (self.max - self.min).into() + 2
    }

    fn size(&self) -> u64 {
        util::log2((self.max - self.min + T::one()).into()) + 1
    }
}
