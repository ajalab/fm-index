use crate::converter::Converter;
use crate::sais;
use crate::suffix_array::SuffixArray;
use crate::util;

use num_traits::Num;
use std::fmt::Debug;
use wavelet_tree::WaveletMatrix;

pub struct FMIndex<T, C, S>
where
    C: Converter<T>,
    S: SuffixArray,
{
    bw: WaveletMatrix,
    occs: Vec<u64>,
    converter: C,
    suffix_array: S,
    _t: std::marker::PhantomData<T>,
}

// TODO: Refactor types (Converter converts T -> u64)
impl<T, C, S> FMIndex<T, C, S>
where
    T: Into<u64> + Copy + Clone + Ord + Num + Debug,
    C: Converter<T>,
    S: SuffixArray,
{
    pub fn new(text: Vec<T>, converter: C, mut suffix_array: S) -> Self {
        let n = text.len();

        let occs = sais::get_bucket_start_pos(&sais::count_chars(&text, &converter));
        let sa = sais::sais(&text, &converter);

        let mut bw = vec![T::zero(); n];
        for i in 0..n {
            let k = sa[i] as usize;
            if k > 0 {
                bw[i] = converter.convert(text[k - 1]);
            }
        }
        let bw = WaveletMatrix::new_with_size(bw, util::log2(converter.len() - 1) + 1);
        suffix_array.build(sa);

        FMIndex {
            occs: occs,
            bw: bw,
            converter: converter,
            suffix_array: suffix_array,
            _t: std::marker::PhantomData::<T>,
        }
    }

    fn get_f_char(&self, i: u64) -> u64 {
        let mut s = 0;
        let mut e = self.occs.len();
        while e - s > 1 {
            let m = s + (e - s) / 2;
            if self.occs[m] <= i {
                s = m;
            } else {
                e = m;
            }
        }
        s as u64
    }

    fn lf_map(&self, c: u64, i: u64) -> u64 {
        let occ = self.occs[c as usize];
        occ + self.bw.rank(c, i)
    }

    fn inverse_lf_map(&self, c: u64, i: u64) -> u64 {
        // binary search to find c s.t. occs[c] <= i < occs[c+1]
        // <=> c is the greatest index s.t. occs[i] <= i
        // invariant: c exists in [s, e)
        let occ = self.occs[c as usize];
        self.bw.select(c, i - occ)
    }

    fn get_sa(&self, mut i: u64) -> u64 {
        let mut steps = 0;
        loop {
            match self.suffix_array.get(i) {
                Some(sa) => {
                    return (sa + steps) % self.bw.len();
                }
                None => {
                    let c = self.bw.access(i);
                    i = self.lf_map(c, i);
                    steps += 1;
                }
            }
        }
    }

    fn search<K>(&self, pattern: K) -> (u64, u64)
    where
        K: AsRef<[T]>,
    {
        let mut s = 0;
        let mut e = self.bw.len();
        for &c in pattern.as_ref().iter().rev() {
            let c = self.converter.convert(c).into();
            s = self.lf_map(c, s);
            e = self.lf_map(c, e);
        }
        (s, e)
    }

    pub fn count<K>(&self, pattern: K) -> u64
    where
        K: AsRef<[T]>,
    {
        let (s, e) = self.search(pattern);
        e - s
    }

    pub fn locate<K>(&self, pattern: K) -> Vec<u64>
    where
        K: AsRef<[T]>,
    {
        let (s, e) = self.search(pattern);
        let mut results: Vec<u64> = Vec::with_capacity((e - s + 1) as usize);
        for k in s..e {
            results.push(self.get_sa(k));
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::RangeConverter;
    use crate::suffix_array::SOSamplingSuffixArray;

    #[test]
    fn test_small() {
        let text = "mississippi\0".to_string().into_bytes();
        let ans = vec![
            ("m", vec![0]),
            ("mi", vec![0]),
            ("m", vec![0]),
            ("i", vec![1, 4, 7, 10]),
            ("iss", vec![1, 4]),
            ("ss", vec![2, 5]),
            ("ss", vec![2, 5]),
            ("p", vec![8, 9]),
            ("ppi", vec![8]),
            ("z", vec![]),
        ];

        let fm_index = FMIndex::new(
            text,
            RangeConverter::new(b'a', b'z'),
            SOSamplingSuffixArray::new(2),
        );

        for (pattern, positions) in ans {
            assert_eq!(fm_index.count(pattern), positions.len() as u64);
            let mut res = fm_index.locate(pattern);
            res.sort();
            assert_eq!(res, positions);
        }
    }

    #[test]
    fn test_utf8() {
        let text = "みんなみんなきれいだな\0"
            .chars()
            .map(|c| c as u32)
            .collect::<Vec<u32>>();
        let ans = vec![
            ("み", vec![0, 3]),
            ("みん", vec![0, 3]),
            ("な", vec![2, 5, 10]),
        ];
        let fm_index = FMIndex::new(
            text,
            RangeConverter::new('あ' as u32, 'ん' as u32),
            SOSamplingSuffixArray::new(2),
        );

        for (pattern, positions) in ans {
            let pattern: Vec<u32> = pattern.chars().map(|c| c as u32).collect();
            assert_eq!(fm_index.count(&pattern), positions.len() as u64);
            let mut res = fm_index.locate(pattern);
            res.sort();
            assert_eq!(res, positions);
        }
    }

    #[test]
    fn test_lf_map() {
        let text = "mississippi\0".to_string().into_bytes();
        let n = text.len();
        let fm_index = FMIndex::new(
            text,
            RangeConverter::new(b'a', b'z'),
            SOSamplingSuffixArray::new(2),
        );
        let mut i = 0;
        for _ in 0..n {
            let c = fm_index.bw.access(i);
            i = fm_index.lf_map(c, i);
        }
    }

    #[test]
    fn test_inverse_lf_map() {
        let text = "mississippi\0".to_string().into_bytes();
        let n = text.len();
        let fm_index = FMIndex::new(
            text,
            RangeConverter::new(b'a', b'z'),
            SOSamplingSuffixArray::new(2),
        );
        let mut i = 0;
        let mut c = fm_index.get_f_char(i);
        for _ in 0..5 {
            println!(
                "char: {} ({})",
                fm_index.converter.convert_inv(c as u8) as char,
                c
            );
            let j = fm_index.inverse_lf_map(c, i);
            c = fm_index.bw.access(i);
            i = j;
            println!("i = {}, c = {}", i, c);
        }
        println!("");
    }

    #[test]
    fn test_extract() {
        let text = "mississippi\0".to_string().into_bytes();
        let n = text.len();
        let fm_index = FMIndex::new(
            text,
            RangeConverter::new(b'a', b'z'),
            SOSamplingSuffixArray::new(2),
        );
        /*
        let i = 3;
        let i = fm_index.get_sa(i);
        let i = fm_index.lf_map(i);
        println!(
            "c = {}",
            fm_index.converter.convert_inv(fm_index.bw.access(i)) as char
        );
        */
    }
}
