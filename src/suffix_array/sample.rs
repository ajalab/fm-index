//! Sampled suffix arrays to perform locate queries.
use crate::util;
use std::fmt;

use serde::{Deserialize, Serialize};
use vers_vecs::BitVec;

/// A sampled suffix array, stored within the index.
#[derive(Serialize, Deserialize)]
pub struct SuffixOrderSampledArray {
    level: usize,
    word_size: usize,
    sa: BitVec,
    len: usize,
}

impl SuffixOrderSampledArray {
    pub(crate) fn get(&self, i: usize) -> Option<usize> {
        if i >= self.len {
            return None;
        }

        if i & ((1 << self.level) - 1) == 0 {
            Some(
                self.sa
                    .get_bits_unchecked((i >> self.level) * self.word_size, self.word_size)
                    as usize,
            )
        } else {
            None
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.sa.heap_size()
    }
}

impl fmt::Debug for SuffixOrderSampledArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.len {
            match self.get(i) {
                Some(sa) => write!(f, "{}", sa)?,
                None => write!(f, "?")?,
            }
        }
        Ok(())
    }
}

impl Default for SuffixOrderSampledArray {
    fn default() -> Self {
        SuffixOrderSampledArray {
            level: 0,
            word_size: 0,
            sa: BitVec::new(),
            len: 0,
        }
    }
}

pub(crate) fn sample(sa: &[usize], mut level: usize) -> SuffixOrderSampledArray {
    if sa.is_empty() {
        return SuffixOrderSampledArray::default();
    }

    let n = sa.len();
    let word_size = (util::log2_usize(n) + 1) as usize;
    if n <= 1 << level {
        // If the sampling level is too high, we don't sample the suffix array at all.
        level = 0;
    }

    let sa_samples_len = ((n - 1) >> level) + 1;
    let mut sa_samples = BitVec::with_capacity(sa_samples_len);
    for i in 0..sa_samples_len {
        sa_samples.append_bits(sa[i << level] as u64, word_size);
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
    fn test_empty() {
        let ssa = sample(&vec![], 2);
        assert_eq!(ssa.get(0), None);
    }

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
            let sa = (0..n).collect::<Vec<usize>>();
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

    #[test]
    fn test_not_sampled() {
        let sa = (0..10).collect::<Vec<usize>>();
        let ssa = sample(&sa, 4);
        for i in 0..10 {
            let v = ssa.get(i);
            assert_eq!(v, Some(i), "ssa[{}] should be Some({})", i, i);
        }
    }
}
