use crate::util;
use std::fmt;

use serde::{Deserialize, Serialize};

pub trait SuffixArray {
    fn get(&self, i: u64) -> Option<u64>;
}

#[derive(Serialize, Deserialize)]
pub struct SOSampledSuffixArray {
    level: usize,
    word_size: usize,
    sa: fid::BitArray,
    len: usize,
}

impl SuffixArray for SOSampledSuffixArray {
    fn get(&self, i: u64) -> Option<u64> {
        if i & ((1 << self.level) - 1) == 0 {
            Some(self.sa.get_word(i as usize >> self.level, self.word_size))
        } else {
            None
        }
    }
}

impl fmt::Debug for SOSampledSuffixArray {
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

pub trait SuffixArraySampler<S: SuffixArray> {
    fn sample(&self, sa: Vec<u64>) -> S;
}

pub struct SuffixArraySOSampler {
    level: usize,
}

impl SuffixArraySOSampler {
    pub fn new() -> Self {
        SuffixArraySOSampler { level: 0 }
    }

    pub fn level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }
}

impl SuffixArraySampler<SOSampledSuffixArray> for SuffixArraySOSampler {
    fn sample(&self, sa: Vec<u64>) -> SOSampledSuffixArray {
        let n = sa.len();
        let word_size = (util::log2(n as u64) + 1) as usize;
        debug_assert!(
            n > (1 << self.level),
            "sampling level L must satisfy 2^L < text_len (L = {}, text_len = {})",
            self.level,
            n,
        );
        let sa_samples_len = n >> self.level;
        let mut sa_samples = fid::BitArray::with_word_size(word_size, sa_samples_len);
        for i in 0..sa_samples_len {
            sa_samples.set_word(i, word_size, sa[i << self.level] as u64);
        }
        SOSampledSuffixArray {
            level: self.level,
            word_size: word_size,
            sa: sa_samples,
            len: sa.len(),
        }
    }
}