use crate::character::Character;

use fid::{BitVector, FID};
use std::fmt;

pub struct WaveletMatrix {
    rows: Vec<BitVector>,
    size: u64,
    len: u64,
    partitions: Vec<u64>,
}

impl WaveletMatrix {
    pub fn new_with_size<T>(text: Vec<T>, size: u64) -> Self
    where
        T: Character,
    {
        let len = text.len() as u64;
        let mut rows: Vec<BitVector> = vec![];
        let mut zeros: Vec<T> = text;
        let mut ones: Vec<T> = Vec::new();
        let mut partitions: Vec<u64> = Vec::new();
        for r in 0..size {
            let mut bv = BitVector::new();
            let mut new_zeros: Vec<T> = Vec::new();
            let mut new_ones: Vec<T> = Vec::new();
            for arr in &[zeros, ones] {
                for &c in arr {
                    let b = c.into();
                    let bit = (b >> (size - r - 1)) & 1 > 0;
                    if bit {
                        new_ones.push(c);
                    } else {
                        new_zeros.push(c);
                    }
                    bv.push(bit);
                }
            }
            zeros = new_zeros;
            ones = new_ones;
            rows.push(bv);
            partitions.push(zeros.len() as u64);
        }
        WaveletMatrix {
            rows: rows,
            size: size,
            len: len,
            partitions: partitions,
        }
    }

    pub fn new<T>(text: Vec<T>) -> Self
    where
        T: Character,
    {
        Self::new_with_size(text, std::mem::size_of::<T>() as u64 * 8)
    }

    pub fn access<T>(&self, k: u64) -> T
    where
        T: Character,
    {
        let mut i = k;
        let mut n = 0u64;
        for (r, bv) in self.rows.iter().enumerate() {
            let b = bv.get(i);
            if b {
                i = self.partitions[r] + bv.rank1(i);
                n = n | (1 << (self.size - (r as u64) - 1));
            } else {
                i = bv.rank0(i);
            }
        }
        Character::from_u64(n)
    }

    pub fn rank<T>(&self, c: T, k: u64) -> u64
    where
        T: Character,
    {
        let n = c.into();
        let mut s = 0u64;
        let mut e = if k < self.len { k } else { self.len };
        for (r, bv) in self.rows.iter().enumerate() {
            let b = (n >> (self.size - (r as u64) - 1)) & 1 > 0;
            s = bv.rank(b, s);
            e = bv.rank(b, e);
            if b {
                let z = self.partitions[r];
                s = s + z;
                e = e + z;
            }
        }
        e - s
    }

    pub fn select<T>(&self, c: T, k: u64) -> u64
    where
        T: Character,
    {
        let n = c.into();
        let mut s = 0u64;
        for (r, bv) in self.rows.iter().enumerate() {
            let b = (n >> (self.size - (r as u64) - 1)) & 1 > 0;
            s = bv.rank(b, s);
            if b {
                let z = self.partitions[r];
                s = s + z;
            }
        }
        let mut e = s + k;
        for (r, bv) in self.rows.iter().enumerate().rev() {
            let b = (n >> (self.size - (r as u64) - 1)) & 1 > 0;
            if b {
                let z = self.partitions[r];
                e = bv.select1(e - z);
            } else {
                e = bv.select0(e);
            }
        }
        e
    }

    pub fn len(&self) -> u64 {
        self.len
    }
}

impl fmt::Debug for WaveletMatrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.rows[0].len();
        writeln!(f, "WaveletMatrix {{")?;
        for bv in &self.rows {
            write!(f, "  ")?;
            for i in 0..len {
                write!(f, "{}", if bv.get(i) { "1" } else { "0" })?;
            }
            writeln!(f, "")?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rank_small() {
        let numbers = vec![4u8, 7, 6, 5, 3, 2, 1, 0, 1, 4, 1, 7];
        let size = 3;
        let wm = WaveletMatrix::new_with_size(numbers.clone(), size);
        assert_eq!(wm.len, numbers.len() as u64);
        for i in 0..(1 << size) {
            let mut r = 0;
            for (k, &n) in numbers.iter().enumerate() {
                assert!(
                    wm.rank(i as u8, k as u64) == r,
                    "wm.rank({}, {}) == {}",
                    i,
                    k,
                    r
                );
                if n == i {
                    r = r + 1;
                }
            }
        }
    }

    #[test]
    fn access_small() {
        let numbers = vec![4u8, 7, 6, 5, 3, 2, 1, 0, 1, 4, 1, 7];
        let size = 3;
        let wm = WaveletMatrix::new_with_size(numbers.clone(), size);
        assert_eq!(wm.len, numbers.len() as u64);
        for (i, &n) in numbers.iter().enumerate() {
            let c: u8 = wm.access(i as u64);
            assert!(c == n, "wm.access({}) == {}", i, n);
        }
    }

    #[test]
    fn select_small() {
        let numbers = vec![4u8, 7, 6, 5, 3, 2, 1, 0, 1, 4, 1, 7];
        let size = 3;
        let wm = WaveletMatrix::new_with_size(numbers.clone(), size);

        let mut ans: Vec<Vec<u64>> = vec![vec![]; 1 << size];
        for (i, &n) in numbers.iter().enumerate() {
            ans[n as usize].push(i as u64);
        }

        for (c, a) in ans.iter().enumerate() {
            for (k, &i) in a.iter().enumerate() {
                assert!(
                    wm.select(c as u8, k as u64) == i,
                    "wm.select({}, {}) == {}",
                    c,
                    k,
                    i
                );
            }
        }
    }

    #[test]
    fn empty() {
        let empty_vec: Vec<u8> = vec![];
        let wm = WaveletMatrix::new(empty_vec);
        assert_eq!(wm.len, 0);
        assert_eq!(wm.rank(0u8, 0), 0);
        assert_eq!(wm.rank(0u8, 10), 0);
        assert_eq!(wm.rank(1u8, 0), 0);
        assert_eq!(wm.rank(1u8, 10), 0);
    }
}
