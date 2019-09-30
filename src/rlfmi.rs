use crate::character::Character;
use crate::converter::{Converter, IndexWithConverter};
use crate::sais;
use crate::search::BackwardIterableIndex;
use crate::suffix_array::{IndexWithSA, SuffixArray, SuffixArraySampler};
use crate::util;
use crate::wavelet_matrix::WaveletMatrix;

use fid::FID;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
}

impl<T, C, S> BackwardIterableIndex for RLFMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    type T = T;

    fn len(&self) -> u64 {
        self.len
    }

    fn get_l(&self, i: u64) -> T {
        // note: b[0] is always 1
        self.s.access(self.b.rank1(i + 1) - 1)
    }

    fn lf_map(&self, i: u64) -> u64 {
        let c = self.get_l(i);
        let nr = self.s.rank(c, self.b.rank1(i));
        self.bp.select1(self.cs[c.into() as usize] + nr) + i - self.b.select1(self.b.rank1(i))
    }

    fn lf_map2(&self, c: T, i: u64) -> u64 {
        let c = self.converter.convert(c);
        let nr = self.s.rank(c, self.b.rank1(i));
        if self.get_l(i) != c {
            self.bp.select1(self.cs[c.into() as usize] + nr)
        } else {
            self.bp.select1(self.cs[c.into() as usize] + nr) + i - self.b.select1(self.b.rank1(i))
        }
    }
}

impl<T, C, S> IndexWithSA for RLFMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
    S: SuffixArray,
{
    fn get_sa(&self, mut i: u64) -> u64 {
        let mut steps = 0;
        loop {
            match self.suffix_array.get(i) {
                Some(sa) => {
                    return (sa + steps) % self.len();
                }
                None => {
                    i = self.lf_map(i);
                    steps += 1;
                }
            }
        }
    }
}

impl<T, C, S> IndexWithConverter<T> for RLFMIndex<T, C, S>
where
    C: Converter<T>,
{
    type C = C;

    fn get_converter(&self) -> &Self::C {
        return &self.converter;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::RangeConverter;
    use crate::suffix_array::{NullSampler, SuffixArraySOSampler};

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
    fn test_locate() {
        let text = "mississippi\0".to_string().into_bytes();
        let ans = vec![
            ("m", vec![0]),
            ("mi", vec![0]),
            ("i", vec![1, 4, 7, 10]),
            ("iss", vec![1, 4]),
            ("ss", vec![2, 5]),
            ("p", vec![8, 9]),
            ("ppi", vec![8]),
            ("z", vec![]),
            ("pps", vec![]),
        ];

        let fm_index = RLFMIndex::new(
            text,
            RangeConverter::new(b'a', b'z'),
            SuffixArraySOSampler::new().level(2),
        );

        for (pattern, positions) in ans {
            let search = fm_index.search_backward(pattern);
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
            let s = rlfmi.lf_map2(c, 0);
            let e = rlfmi.lf_map2(c, n);
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

    #[test]
    fn test_search_backward() {
        let text = "mississippi\0".to_string().into_bytes();
        let ans = vec![
            ("iss", (3, 5)),
            ("ppi", (7, 8)),
            ("si", (8, 10)),
            ("ssi", (10, 12)),
        ];
        let rlfmi = RLFMIndex::new(text, RangeConverter::new(b'a', b'z'), NullSampler::new());

        for (s, r) in ans {
            let search = rlfmi.search_backward(s);
            assert_eq!(search.get_range(), r);
        }
    }

    #[test]
    fn test_iter_backward() {
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\0".to_string().into_bytes();
        let rlfmi = RLFMIndex::new(text, RangeConverter::new(b' ', b'~'), NullSampler::new());
        let search = rlfmi.search_backward("sit ");
        println!("{:?}", search.get_range());
        let mut prev_seq = rlfmi
            .iter_backward(search.get_range().0)
            .take(6)
            .collect::<Vec<_>>();
        prev_seq.reverse();
        assert_eq!(prev_seq, b"dolor ".to_owned());
    }
}
