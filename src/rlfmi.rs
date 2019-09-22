use crate::character::Character;
use crate::converter::Converter;
use crate::sais;
use crate::suffix_array::SuffixArraySampler;
use crate::util;
use crate::wavelet_matrix::WaveletMatrix;

use fid::FID;

pub struct RLFMIndex<T, C, S> {
    converter: C,
    suffix_array: S,
    s: WaveletMatrix,
    b: fid::BitVector,
    bp: fid::BitVector,
    cs: Vec<u64>,
    len: u64,
    _t: std::marker::PhantomData<T>,
}

impl<T, C, S> RLFMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    pub fn new<B: SuffixArraySampler<S>>(text: Vec<T>, converter: C, sampler: B) -> Self {
        let n = text.len();
        let m = converter.len();
        let sa = sais::sais(&text, &converter);

        let mut c0 = T::zero();
        // sequence of run heads
        let mut s = Vec::new();
        // sequence of run lengths
        // run length `l` is encoded as 10^{l-1}
        let mut b = fid::BitVector::new();
        let mut runs_by_char: Vec<Vec<usize>> = vec![vec![]; m as usize];
        for i in 0..n {
            let k = sa[i] as usize;
            let c = converter.convert(if k > 0 { text[k - 1] } else { text[n - 1] });
            // We do not allow consecutive occurrences of zeroes,
            // so text[sa[0] - 1] = text[n - 2] is not zero.
            if c0 != c {
                s.push(c);
                b.push(true);
                runs_by_char[c.into() as usize].push(1);
            } else {
                b.push(false);
                match runs_by_char[c.into() as usize].last_mut() {
                    Some(r) => *r += 1,
                    None => unreachable!(),
                };
            }
            c0 = c;
        }
        let s = WaveletMatrix::new_with_size(s, util::log2(m - 1) + 1);
        let mut bp = fid::BitVector::new();
        let mut cs = vec![0u64; m as usize];
        let mut c = 0;
        for (rs, ci) in runs_by_char.into_iter().zip(&mut cs) {
            *ci = c;
            c = c + rs.len() as u64;
            for r in rs {
                bp.push(true);
                for _ in 0..(r - 1) {
                    bp.push(false);
                }
            }
        }

        RLFMIndex {
            converter: converter,
            suffix_array: sampler.sample(sa),
            s: s,
            b: b,
            bp: bp,
            cs: cs,
            len: n as u64,
            _t: std::marker::PhantomData::<T>,
        }
    }

    fn get_l(&self, i: u64) -> u64 {
        // note: b[0] is always 1
        self.s.access::<u64>(self.b.rank1(i + 1) - 1)
    }

    fn lf_map(&self, i: u64) -> u64 {
        let c = self.get_l(i);
        let nr = self.s.rank(c, self.b.rank1(i));
        self.bp.select1(self.cs[c as usize] + nr) + i - self.b.select1(self.b.rank1(i))
    }

    fn lf_map2(&self, c: u64, i: u64) -> u64 {
        let nr = self.s.rank(c, self.b.rank1(i));
        if self.get_l(i) != c {
            self.bp.select1(self.cs[c as usize] + nr)
        } else {
            self.bp.select1(self.cs[c as usize] + nr) + i - self.b.select1(self.b.rank1(i))
        }
    }

    fn len(&self) -> u64 {
        self.len
    }

    pub fn search_backward<'a, K>(&'a self, pattern: K) -> Search<'a, T, C, S>
    where
        K: AsRef<[T]>,
    {
        Search::new(self, 0, self.len, vec![]).search_backward(pattern)
    }
}

pub struct Search<'a, T, C, S>
where
    C: Converter<T>,
{
    fm_index: &'a RLFMIndex<T, C, S>,
    s: u64,
    e: u64,
    pattern: Vec<T>,
}

impl<'a, T, C, S> Search<'a, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    fn new(fm_index: &'a RLFMIndex<T, C, S>, s: u64, e: u64, pattern: Vec<T>) -> Self {
        Search {
            fm_index: fm_index,
            s: s,
            e: e,
            pattern: pattern,
        }
    }

    pub fn get_range(&self) -> (u64, u64) {
        (self.s, self.e)
    }

    pub fn search_backward<K: AsRef<[T]>>(&self, pattern: K) -> Self {
        let mut s = self.s;
        let mut e = self.e;
        let mut pattern = pattern.as_ref().to_owned();
        for &c in pattern.iter().rev() {
            let c = self.fm_index.converter.convert(c).into();
            s = self.fm_index.lf_map2(c, s);
            e = self.fm_index.lf_map2(c, e);
            if s == e {
                break;
            }
        }
        pattern.extend_from_slice(&self.pattern);

        Search {
            fm_index: self.fm_index,
            s: s,
            e: e,
            pattern: pattern,
        }
    }

    pub fn count(&self) -> u64 {
        self.e - self.s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::RangeConverter;
    use crate::suffix_array::NullSampler;

    use fid::FID;

    #[test]
    fn test_count() {
        let text = "mississippi\0".to_string().into_bytes();
        let ans = vec![
            ("m", 1),
            ("mi", 1),
            ("i", 4),
            ("iss", 2),
            ("ss", 2),
            ("p", 2),
            ("ppi", 1),
            ("z", 0),
            ("pps", 0),
        ];
        let rlfmi = RLFMIndex::new(text, RangeConverter::new(b'a', b'z'), NullSampler::new());
        for (pattern, expected) in ans {
            let search = rlfmi.search_backward(pattern);
            let actual = search.count();
            assert_eq!(
                expected, actual,
                "pattern \"{}\" must occur {} times, but {}",
                pattern, expected, actual,
            );
        }
    }

    #[test]
    fn test_s() {
        let text = "mississippi\0".to_string().into_bytes();
        let rlfmi = RLFMIndex::new(text, RangeConverter::new(b'a', b'z'), NullSampler::new());
        let ans = "ipsm\0pisi".to_string().into_bytes();
        for (i, a) in ans.into_iter().enumerate() {
            let l: u8 = rlfmi.s.access(i as u64);
            assert_eq!(rlfmi.converter.convert_inv(l), a);
        }
    }

    #[test]
    fn test_b() {
        let text = "mississippi\0".to_string().into_bytes();
        let rlfmi = RLFMIndex::new(text, RangeConverter::new(b'a', b'z'), NullSampler::new());
        let n = rlfmi.len();
        let ans = vec![1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0];
        // l:      ipssm$pissii
        // b:      111011111010
        // rank_0  0123345678899
        // rank_1  1233456788999
        // s:      ipsm$pisi
        //         012345678
        assert_eq!(n, rlfmi.b.len());
        for (i, a) in ans.into_iter().enumerate() {
            assert_eq!(rlfmi.b.get(i as u64), a != 0, "failed at b[{}]", i);
        }
    }

    #[test]
    fn test_bp() {
        let text = "mississippi\0".to_string().into_bytes();
        let rlfmi = RLFMIndex::new(text, RangeConverter::new(b'a', b'z'), NullSampler::new());
        let n = rlfmi.len();
        let ans = vec![1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0];
        assert_eq!(n, rlfmi.bp.len());
        for (i, a) in ans.into_iter().enumerate() {
            assert_eq!(rlfmi.bp.get(i as u64), a != 0, "failed at bp[{}]", i);
        }
    }

    #[test]
    fn test_cs() {
        let text = "mississippi\0".to_string().into_bytes();
        let rlfmi = RLFMIndex::new(text, RangeConverter::new(b'a', b'z'), NullSampler::new());
        let ans = vec![(b'\0', 0), (b'i', 1), (b'm', 4), (b'p', 5), (b's', 7)];
        for (c, a) in ans {
            let c = rlfmi.converter.convert(c) as usize;
            assert_eq!(rlfmi.cs[c], a);
        }
    }

    #[test]
    fn test_get_l() {
        let text = "mississippi\0".to_string().into_bytes();
        let rlfmi = RLFMIndex::new(text, RangeConverter::new(b'a', b'z'), NullSampler::new());
        let ans = "ipssm\0pissii".to_string().into_bytes();

        for (i, a) in ans.into_iter().enumerate() {
            let l = rlfmi.get_l(i as u64);
            assert_eq!(rlfmi.converter.convert_inv(l as u8), a);
        }
    }

    #[test]
    fn test_lf_map() {
        let text = "mississippi\0".to_string().into_bytes();
        let n = text.len();
        let ans = [1, 6, 7, 2, 8, 10, 3, 9, 11, 4, 5, 0];
        let rlfmi = RLFMIndex::new(text, RangeConverter::new(b'a', b'z'), NullSampler::new());

        let mut i = 0;
        for k in 0..n {
            let next_i = rlfmi.lf_map(i);
            assert_eq!(next_i, ans[k], "should be lf_map({}) == {}", i, ans[k]);
            i = next_i;
        }
    }

    #[test]
    fn test_lf_map2() {
        let text = "mississippi\0".to_string().into_bytes();
        let n = text.len() as u64;
        let ans = vec![
            (b'\0', (0, 1)),
            (b'i', (1, 5)),
            (b'm', (5, 6)),
            (b'p', (6, 8)),
            (b's', (8, 12)),
        ];
        let rlfmi = RLFMIndex::new(text, RangeConverter::new(b'a', b'z'), NullSampler::new());

        for (c, r) in ans {
            let cc = rlfmi.converter.convert(c).into();
            let s = rlfmi.lf_map2(cc, 0);
            let e = rlfmi.lf_map2(cc, n);
            assert_eq!(
                (s, e),
                r,
                "character {:?}: (s, e) should be {:?}, actual: {:?}",
                c as char,
                r,
                (s, e)
            );
        }
    }
}
