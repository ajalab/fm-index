//! Suffix arrays, used to construct the index.
//!
//! Can also be used in sampled fashion to perform locate queries.
use crate::util;
use std::fmt;

use serde::{Deserialize, Serialize};
use vers_vecs::BitVec;

pub trait IndexWithSA {
    fn get_sa(&self, i: u64) -> u64;
}

// pub(crate) trait PartialArray {
//     fn get(&self, i: u64) -> Option<u64>;
//     fn size(&self) -> usize;
// }

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

/// In order to perform locate queries, we need to retain the suffix array that
/// is generated during the construction phase. But we do not need the whole
/// array as we can interpolate missing elements in a suffix array from other
/// elements.
///
/// A sampler will _sieve_ a suffix array for this purpose.
pub trait ArraySampler<S> {
    /// Given a suffix array, sample it and create a sampled array.
    fn sample(&self, sa: Vec<u64>) -> S;
}

/// The `NullSampler` does not store any sampled information.
///
/// If you do not need `locate` queries you can use this sampler.
/// You won't have access to `locate` on the type level.
#[derive(Default)]
pub struct NullSampler {}

impl NullSampler {
    /// Construct a new null sampler.
    pub fn new() -> Self {
        NullSampler {}
    }
}

impl ArraySampler<()> for NullSampler {
    fn sample(&self, _sa: Vec<u64>) {}
}

/// A sampler that sieves the suffix array for information to retain.
///
/// Use this if you want to perform `locate` queries.
#[derive(Default)]
pub struct SuffixOrderSampler {
    level: usize,
}

impl SuffixOrderSampler {
    /// Construct a new suffix order sampler.
    ///
    /// Defaults to level 0, meaning the information in the suffix array
    /// is completely retained, with `O(N)` space complexity where `N`
    /// is the size of the text.
    pub fn new() -> Self {
        SuffixOrderSampler { level: 0 }
    }

    /// Set the sampling level.
    ///
    /// The sampling level can be used to reduce the amount of information
    /// retained at the cost of run-time performance.
    ///
    /// The sampling level `L` affects the space complexity of the sampled
    /// array as `O(N / 2^L)` where `N` is the size of the text.
    ///
    /// The sampling level `L` must satisfy `2^L < N`.
    pub fn level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }
}

impl ArraySampler<SuffixOrderSampledArray> for SuffixOrderSampler {
    fn sample(&self, sa: Vec<u64>) -> SuffixOrderSampledArray {
        let n = sa.len();
        let word_size = (util::log2(n as u64) + 1) as usize;
        debug_assert!(n > 0);
        debug_assert!(
            n > (1 << self.level),
            "sampling level L must satisfy 2^L < text_len (L = {}, text_len = {})",
            self.level,
            n,
        );
        let sa_samples_len = ((n - 1) >> self.level) + 1;
        let mut sa_samples = BitVec::with_capacity(sa_samples_len);
        // fid::BitArray::with_word_size(word_size, sa_samples_len);
        for i in 0..sa_samples_len {
            sa_samples.append_bits(sa[i << self.level], word_size);
        }
        SuffixOrderSampledArray {
            level: self.level,
            word_size,
            sa: sa_samples,
            len: sa.len(),
        }
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
            let ssa = SuffixOrderSampler::new().level(level).sample(sa);
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
