use crate::character::Character;
use crate::converter::Converter;
use crate::sais;
use crate::suffix_array::SuffixArray;
use crate::util;
use crate::wavelet_matrix::WaveletMatrix;

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
    T: Character,
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

    pub fn search<'a, K>(&'a self, pattern: K) -> Search<'a, T, C, S>
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
        Search::new(self, s, e, pattern.as_ref().to_vec())
    }
}

pub struct Search<'a, T, C, S>
where
    C: Converter<T>,
    S: SuffixArray,
{
    fm_index: &'a FMIndex<T, C, S>,
    s: u64,
    e: u64,
    pattern: Vec<T>,
}

impl<'a, T, C, S> Search<'a, T, C, S>
where
    T: Character,
    C: Converter<T>,
    S: SuffixArray,
{
    fn new(fm_index: &'a FMIndex<T, C, S>, s: u64, e: u64, pattern: Vec<T>) -> Self {
        Search {
            fm_index: fm_index,
            s: s,
            e: e,
            pattern: pattern,
        }
    }

    pub fn count(&self) -> u64 {
        self.e - self.s
    }

    pub fn locate(&self) -> Vec<u64> {
        let mut results: Vec<u64> = Vec::with_capacity((self.e - self.s + 1) as usize);
        for k in self.s..self.e {
            results.push(self.fm_index.get_sa(k));
        }
        results
    }

    pub fn display_prefix(&self, i: usize, l: usize) -> Vec<T> {
        let mut result = Vec::with_capacity(l);
        let mut i = self.s + i as u64;
        debug_assert!(i < self.e);
        for _ in 0..l {
            let c: T = self.fm_index.bw.access(i);
            if c.into() == 0 {
                break;
            }
            result.push(self.fm_index.converter.convert_inv(c));
            i = self.fm_index.lf_map(c.into(), i);
        }
        result.reverse();
        result
    }

    pub fn display_postfix(&self, i: usize, n: usize, r: usize) -> Vec<T> {
        let mut result = Vec::with_capacity(r);
        let mut i = self.s + i as u64;
        debug_assert!(i < self.e);
        for _ in 0..n {
            let c = self.fm_index.get_f_char(i);
            i = self.fm_index.inverse_lf_map(c, i);
        }
        for _ in 0..r {
            let c = self.fm_index.get_f_char(i);
            if c == 0 {
                break;
            }
            i = self.fm_index.inverse_lf_map(c, i);
            result.push(self.fm_index.converter.convert_inv(Character::from_u64(c)));
        }
        result
    }

    pub fn display(&self, i: usize, l: usize, r: usize) -> Vec<T> {
        let mut result = Vec::with_capacity(l + self.pattern.len() + r);
        let mut prefix = self.display_prefix(i, l);
        let mut postfix = self.display_postfix(i, self.pattern.len(), r);
        result.append(&mut prefix);
        result.extend(&self.pattern);
        result.append(&mut postfix);
        result
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
            let search = fm_index.search(pattern);
            let expected = positions.len() as u64;
            let actual = search.count();
            assert_eq!(
                expected,
                actual,
                "pattern \"{}\" must occur {} times, but {}: {:?}",
                pattern,
                expected,
                actual,
                search.locate()
            );
            let mut res = search.locate();
            res.sort();
            assert_eq!(res, positions);
        }
    }

    #[test]
    fn test_small_contain_null() {
        let text = "miss\0issippi\0".to_string().into_bytes();
        let fm_index = FMIndex::new(
            text,
            RangeConverter::new(b'a', b'z'),
            SOSamplingSuffixArray::new(2),
        );
        assert_eq!(fm_index.search("m").count(), 1);
        assert_eq!(fm_index.search("ssi").count(), 1);
        assert_eq!(fm_index.search("iss").count(), 2);
        assert_eq!(fm_index.search("p").count(), 2);
        assert_eq!(fm_index.search("\0").count(), 2);
        assert_eq!(fm_index.search("\0i").count(), 1);
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
            let search = fm_index.search(pattern);
            assert_eq!(search.count(), positions.len() as u64);
            let mut res = search.locate();
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
        let fm_index = FMIndex::new(
            text,
            RangeConverter::new(b'a', b'z'),
            SOSamplingSuffixArray::new(2),
        );
        let cases = vec![5u64, 0, 7, 10, 11, 4, 1, 6, 2, 3, 8, 9];
        for (i, expected) in cases.into_iter().enumerate() {
            let c = fm_index.get_f_char(i as u64);
            let actual = fm_index.inverse_lf_map(c, i as u64);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_display() {
        let text = "mississippi\0".to_string().into_bytes();
        let fm_index = FMIndex::new(
            text,
            RangeConverter::new(b'a', b'z'),
            SOSamplingSuffixArray::new(2),
        );
        let search = fm_index.search("ssi");
        assert_eq!(search.display(0, 2, 2), "sissipp".to_owned().as_bytes());
        assert_eq!(search.display(1, 2, 2), "mississ".to_owned().as_bytes());
    }
}
