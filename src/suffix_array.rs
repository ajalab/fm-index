//! Suffix arrays, used to construct the index.
//!
//! Can also be used in sampled fashion to perform locate queries.
use crate::{seal, util};
use std::fmt;

use serde::{Deserialize, Serialize};
use vers_vecs::BitVec;

/// A trait for an index that supports locate queries.
///
/// This is only supported when [`SuffixOrderSampledArray`] is passed in.
pub trait HasPosition {
    #[doc(hidden)]
    fn get_sa<L: seal::IsLocal>(&self, i: u64) -> u64;
}

/// A sampled suffix array, stored within the index.
#[derive(Serialize, Deserialize)]
pub struct SuffixOrderSampledArray {
    level: usize,
    word_size: usize,
    sa: BitVec,
    len: usize,
}

impl SuffixOrderSampledArray {
    pub(crate) fn get(&self, i: u64) -> Option<u64> {
        debug_assert!(i < self.len as u64);
        if i & ((1 << self.level) - 1) == 0 {
            Some(
                self.sa.get_bits_unchecked(
                    (i as usize >> self.level) * self.word_size,
                    self.word_size,
                ),
            )
        } else {
            None
        }
    }

    pub(crate) fn size(&self) -> usize {
        std::mem::size_of::<Self>() + self.sa.heap_size()
    }
}

impl fmt::Debug for SuffixOrderSampledArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.len {
            match self.get(i as u64) {
                Some(sa) => write!(f, "{}", sa)?,
                None => write!(f, "?")?,
            }
        }
        Ok(())
    }
}

pub(crate) fn sample(sa: &[u64], level: usize) -> SuffixOrderSampledArray {
    let n = sa.len();
    let word_size = (util::log2(n as u64) + 1) as usize;
    debug_assert!(n > 0);
    debug_assert!(
        n > (1 << level),
        "sampling level L must satisfy 2^L < text_len (L = {}, text_len = {})",
        level,
        n,
    );
    let sa_samples_len = ((n - 1) >> level) + 1;
    let mut sa_samples = BitVec::with_capacity(sa_samples_len);
    // fid::BitArray::with_word_size(word_size, sa_samples_len);
    for i in 0..sa_samples_len {
        sa_samples.append_bits(sa[i << level], word_size);
    }
    SuffixOrderSampledArray {
        level,
        word_size,
        sa: sa_samples,
        len: sa.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular() {
        let cases = [
            (1, 10),
            (1, 25),
            (2, 8),
            (2, 9),
            (2, 10),
            (2, 25),
            (3, 24),
            (3, 25),
        ];
        for &(level, n) in cases.iter() {
            let sa = (0..n).collect::<Vec<u64>>();
            let ssa = sample(&sa, level);
            for i in 0..n {
                let v = ssa.get(i);
                if i & ((1 << level) - 1) == 0 {
                    assert_eq!(v, Some(i), "ssa[{}] should be Some({})", i, i);
                } else {
                    assert_eq!(v, None, "ssa[{}] should be None", i);
                }
            }
        }
    }
}
