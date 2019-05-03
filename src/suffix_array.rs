use crate::util;
use std::fmt;

pub trait SuffixArray {
    fn build(&mut self, sa: Vec<usize>);
    fn get(&self, i: u64) -> Option<u64>;
}

pub struct SOSamplingSuffixArray {
    level: usize,
    word_size: usize,
    sa: fid::BitArray,
    len: usize,
}

impl SOSamplingSuffixArray {
    pub fn new(level: usize) -> Self {
        SOSamplingSuffixArray {
            level: level,
            word_size: 0,
            sa: fid::BitArray::new(0),
            len: 0,
        }
    }
}

impl fmt::Debug for SOSamplingSuffixArray {
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

impl SuffixArray for SOSamplingSuffixArray {
    fn build(&mut self, sa: Vec<usize>) {
        let n = sa.len();
        let word_size = (util::log2(n as u64) + 1) as usize;
        debug_assert!(
            n > (1 << self.level),
            "sampling level L must satisfy 2^L < text_len"
        );
        let sa_samples_len = n >> self.level;
        let mut sa_samples = fid::BitArray::with_word_size(word_size, sa_samples_len);
        for i in 0..sa_samples_len {
            sa_samples.set_word(i, word_size, sa[i << self.level] as u64);
        }
        self.word_size = word_size;
        self.sa = sa_samples;
        self.len = sa.len();
    }

    fn get(&self, i: u64) -> Option<u64> {
        if i & ((1 << self.level) - 1) == 0 {
            Some(self.sa.get_word(i as usize >> self.level, self.word_size))
        } else {
            None
        }
    }
}
