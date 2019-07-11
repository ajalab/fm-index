use crate::character::Character;

use serde::{Deserialize, Serialize};

pub trait Converter<T> {
    fn convert(&self, c: T) -> T;
    fn convert_inv(&self, c: T) -> T;
    fn len(&self) -> u64;
}

#[derive(Serialize, Deserialize)]
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

pub struct IdConverter {
    size: u64,
}

impl IdConverter {
    pub fn new(size: u64) -> Self {
        IdConverter { size: size }
    }
}

impl<T> Converter<T> for IdConverter {
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
