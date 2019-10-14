use crate::util;
use std::fmt;

use serde::{Deserialize, Serialize};

pub trait IndexWithSA {
    fn get_sa(&self, i: u64) -> u64;
}

pub trait PartialArray {
    fn get(&self, i: u64) -> Option<u64>;
}

#[derive(Serialize, Deserialize)]
pub struct RegularSampledArray {
    level: usize,
    word_size: usize,
    sa: fid::BitArray,
    len: usize,
}

impl PartialArray for RegularSampledArray {
    fn get(&self, i: u64) -> Option<u64> {
        debug_assert!(i < self.len as u64);
        if i & ((1 << self.level) - 1) == 0 {
            Some(self.sa.get_word(i as usize >> self.level, self.word_size))
        } else {
            None
        }
    }
}

impl fmt::Debug for RegularSampledArray {
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

pub trait ArraySampler<S> {
    fn sample(&self, sa: Vec<u64>) -> S;
}

#[derive(Default)]
pub struct NullSampler {}

impl NullSampler {
    pub fn new() -> Self {
        NullSampler {}
    }
}

impl ArraySampler<()> for NullSampler {
    fn sample(&self, _sa: Vec<u64>) {}
}

#[derive(Default)]
pub struct RegularSampler {
    level: usize,
}

impl RegularSampler {
    pub fn new() -> Self {
        RegularSampler { level: 0 }
    }

    pub fn level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }
}

impl ArraySampler<RegularSampledArray> for RegularSampler {
    fn sample(&self, sa: Vec<u64>) -> RegularSampledArray {
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
        let mut sa_samples = fid::BitArray::with_word_size(word_size, sa_samples_len);
        for i in 0..sa_samples_len {
            sa_samples.set_word(i, word_size, sa[i << self.level] as u64);
        }
        RegularSampledArray {
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
            let ssa = RegularSampler::new().level(level).sample(sa);
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
