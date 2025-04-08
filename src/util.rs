use std::ops::{Rem, Sub};

pub fn log2(x: u64) -> u64 {
    ((std::mem::size_of::<u64>() * 8) as u64) - u64::from(x.leading_zeros()) - 1
}

pub fn log2_usize(x: usize) -> usize {
    ((std::mem::size_of::<usize>() * 8) as usize) - (x.leading_zeros() as usize) - 1
}

pub fn modular_add<T: Rem<Output = T> + Ord + num_traits::Zero>(a: T, b: T, m: T) -> T {
    debug_assert!(T::zero() <= a && a <= m);
    debug_assert!(T::zero() <= b && b <= m);

    (a + b) % m
}

pub fn modular_sub<T: Sub<Output = T> + Ord + num_traits::Zero>(a: T, b: T, m: T) -> T {
    debug_assert!(T::zero() <= a && a <= m);
    debug_assert!(T::zero() <= b && b <= m);

    if a >= b {
        a - b
    } else {
        m + a - b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_log2() {
        assert_eq!(log2(2u64), 1);
        assert_eq!(log2(3u64), 1);
        assert_eq!(log2(4u64), 2);
        assert_eq!(log2(5u64), 2);
        assert_eq!(log2(6u64), 2);
        assert_eq!(log2(7u64), 2);
        assert_eq!(log2(8u64), 3);
    }
}
