use crate::backend::{HasPosition, HeapSize, SearchIndexBackend};
#[cfg(doc)]
use crate::converter;
use crate::converter::Converter;
use crate::suffix_array::sais;
use crate::suffix_array::sample::SuffixOrderSampledArray;
use crate::util;

use serde::{Deserialize, Serialize};
use vers_vecs::{BitVec, RsVec, WaveletMatrix};

/// A Run-Length FM-index.
///
/// This can be more space-efficient than the FM-index, but is slower.
#[derive(Serialize, Deserialize)]
pub struct RLFMIndexBackend<T, C, S> {
    converter: C,
    suffix_array: S,
    s: WaveletMatrix,
    b: RsVec,
    bp: RsVec,
    cs: Vec<usize>,
    len: usize,
    _t: std::marker::PhantomData<T>,
}

impl<T, C, S> RLFMIndexBackend<T, C, S>
where
    T: Copy + Clone,
    C: Converter<Char = T>,
{
    pub(crate) fn new(text: &[T], converter: C, get_sample: impl Fn(&[usize]) -> S) -> Self {
        let n = text.len();
        let m = converter.to_usize(converter.max_value()) + 1;
        let sa = sais::build_suffix_array(text, &converter);

        let mut c0 = 0;
        // sequence of run heads
        let mut s = Vec::new();
        // sequence of run lengths
        // run length `l` is encoded as 10^{l-1}
        let mut b = BitVec::new();
        let mut runs_by_char: Vec<Vec<usize>> = vec![vec![]; m];
        for &k in &sa {
            let c = if k > 0 { text[k - 1] } else { text[n - 1] };
            // We do not allow consecutive occurrences of zeroes,
            // so text[sa[0] - 1] = text[n - 2] is not zero.
            if c0 != converter.to_u64(c) {
                s.push(converter.to_u64(c));
                b.append(true);
                runs_by_char[converter.to_usize(c)].push(1);
            } else {
                b.append(false);
                match runs_by_char[converter.to_usize(c)].last_mut() {
                    Some(r) => *r += 1,
                    None => unreachable!(),
                };
            }
            c0 = converter.to_u64(c);
        }
        let s = WaveletMatrix::from_slice(&s, (util::log2(m as u64 - 1) + 1) as u16);
        let mut bp = BitVec::new();
        let mut cs = vec![0usize; m];
        let mut c = 0;
        for (rs, ci) in runs_by_char.into_iter().zip(&mut cs) {
            *ci = c;
            c += rs.len();
            for r in rs {
                bp.append(true);
                for _ in 0..(r - 1) {
                    bp.append(false);
                }
            }
        }

        let b = RsVec::from_bit_vec(b);
        let bp = RsVec::from_bit_vec(bp);
        RLFMIndexBackend {
            converter,
            suffix_array: get_sample(&sa),
            s,
            b,
            bp,
            cs,
            len: n,
            _t: std::marker::PhantomData::<T>,
        }
    }
}

impl<T, C> HeapSize for RLFMIndexBackend<T, C, ()>
where
    T: Copy + Clone,
    C: Converter<Char = T>,
{
    fn heap_size(&self) -> usize {
        self.s.heap_size()
            + self.b.heap_size()
            + self.bp.heap_size()
            + self.cs.capacity() * std::mem::size_of::<usize>()
    }
}

impl<T, C> HeapSize for RLFMIndexBackend<T, C, SuffixOrderSampledArray>
where
    T: Copy + Clone,
    C: Converter<Char = T>,
{
    fn heap_size(&self) -> usize {
        self.s.heap_size()
            + self.b.heap_size()
            + self.bp.heap_size()
            + self.cs.capacity() * std::mem::size_of::<usize>()
            + self.suffix_array.size()
    }
}

impl<T, C, S> SearchIndexBackend for RLFMIndexBackend<T, C, S>
where
    T: Copy + Clone,
    C: Converter<Char = T>,
{
    type T = T;
    type C = C;

    fn len(&self) -> usize {
        self.len
    }

    fn get_l(&self, i: usize) -> T {
        // note: b[0] is always 1
        self.converter
            .from_u64(self.s.get_u64_unchecked(self.b.rank1(i + 1) - 1))
    }

    fn lf_map(&self, i: usize) -> usize {
        let c = self.get_l(i);
        let j = self.b.rank1(i);
        let nr = self.s.rank_u64_unchecked(j, self.converter.to_u64(c));

        self.bp.select1(self.cs[self.converter.to_usize(c)] + nr) + i - self.b.select1(j)
    }

    fn lf_map2(&self, c: T, i: usize) -> usize {
        let c_u64 = self.converter.to_u64(c);
        let c_usize = self.converter.to_usize(c);
        let j = self.b.rank1(i);
        let nr = self.s.rank_u64_unchecked(j, c_u64);
        if self.converter.to_u64(self.get_l(i)) != c_u64 {
            self.bp.select1(self.cs[c_usize] + nr)
        } else {
            self.bp.select1(self.cs[c_usize] + nr) + i - self.b.select1(j)
        }
    }

    fn get_f(&self, i: usize) -> Self::T {
        let mut s = 0;
        let mut e = self.cs.len();
        let r = self.bp.rank1(i + 1) - 1;
        while e - s > 1 {
            let m = s + (e - s) / 2;
            if self.cs[m] <= r {
                s = m;
            } else {
                e = m;
            }
        }
        self.converter.from_usize(s)
    }

    fn fl_map(&self, i: usize) -> Option<usize> {
        let c = self.get_f(i);
        let c_u64 = self.converter.to_u64(c);
        let c_usize = self.converter.to_usize(c);
        let j = self.bp.rank1(i + 1) - 1;
        let p = self.bp.select1(j);
        let m = self.s.select_u64_unchecked(j - self.cs[c_usize], c_u64);
        let n = self.b.select1(m);
        Some(n + i - p)
    }

    fn get_converter(&self) -> &Self::C {
        &self.converter
    }
}

impl<T, C> HasPosition for RLFMIndexBackend<T, C, SuffixOrderSampledArray>
where
    T: Copy + Clone,
    C: Converter<Char = T>,
{
    fn get_sa(&self, mut i: usize) -> usize {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{converter::NoOpConverter, wrapper::SearchIndexWrapper};

    #[test]
    fn test_s() {
        let text = "mississippi\0".as_bytes();
        let rlfmi = RLFMIndexBackend::new(text, NoOpConverter::<u8>::default(), |_| ());
        let ans = "ipsm\0pisi".as_bytes();
        for (i, a) in ans.iter().enumerate() {
            let l = rlfmi.s.get_u64_unchecked(i);
            assert_eq!(rlfmi.converter.from_u64(l), *a);
        }
    }

    #[test]
    fn test_b() {
        let text = "mississippi\0".as_bytes();
        let rlfmi = RLFMIndexBackend::new(text, NoOpConverter::<u8>::default(), |_| ());
        let n = rlfmi.len();
        let ans = vec![1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0];
        // l:      ipssm$pissii
        // b:      111011111010
        // rank_0  0123345678899
        // rank_1  1233456788999
        // s:      ipsm$pisi
        //         012345678
        assert_eq!({ n }, rlfmi.b.len());
        for (i, a) in ans.into_iter().enumerate() {
            assert_eq!(
                rlfmi.b.get(i).unwrap(),
                (a != 0) as u64,
                "failed at b[{}]",
                i
            );
        }
    }

    #[test]
    fn test_bp() {
        let text = "mississippi\0".as_bytes();
        let rlfmi = RLFMIndexBackend::new(text, NoOpConverter::<u8>::default(), |_| ());
        let n = rlfmi.len();
        let ans = vec![1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0];
        assert_eq!({ n }, rlfmi.bp.len());
        for (i, a) in ans.into_iter().enumerate() {
            assert_eq!(
                rlfmi.bp.get(i).unwrap(),
                (a != 0) as u64,
                "failed at bp[{}]",
                i
            );
        }
    }

    #[test]
    fn test_cs() {
        let text = "mississippi\0".as_bytes();
        let rlfmi = RLFMIndexBackend::new(text, NoOpConverter::<u8>::default(), |_| ());
        let ans = vec![(b'\0', 0), (b'i', 1), (b'm', 4), (b'p', 5), (b's', 7)];
        for (c, a) in ans {
            let c = rlfmi.converter.to_usize(c);
            assert_eq!(rlfmi.cs[c], a);
        }
    }

    #[test]
    fn test_get_l() {
        let text = "mississippi\0".as_bytes();
        let rlfmi = RLFMIndexBackend::new(text, NoOpConverter::<u8>::default(), |_| ());
        let ans = "ipssm\0pissii".to_string().into_bytes();

        for (i, a) in ans.into_iter().enumerate() {
            let l = rlfmi.get_l(i);
            assert_eq!(l, a);
        }
    }

    #[test]
    fn test_lf_map() {
        let text = "mississippi\0".as_bytes();
        let ans = vec![1, 6, 7, 2, 8, 10, 3, 9, 11, 4, 5, 0];
        let rlfmi = RLFMIndexBackend::new(text, NoOpConverter::<u8>::default(), |_| ());

        let mut i = 0;
        for a in ans {
            let next_i = rlfmi.lf_map(i);
            assert_eq!(next_i, a, "should be lf_map({}) == {}", i, a);
            i = next_i;
        }
    }

    #[test]
    fn test_lf_map2() {
        let text = "mississippi\0".as_bytes();
        let ans = vec![
            (b'\0', (0, 1)),
            (b'i', (1, 5)),
            (b'm', (5, 6)),
            (b'p', (6, 8)),
            (b's', (8, 12)),
        ];
        let rlfmi = RLFMIndexBackend::new(text, NoOpConverter::<u8>::default(), |_| ());
        let n = rlfmi.len();

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
        let text = "mississippi\0".as_bytes();
        let ans = vec![
            ("iss", (3, 5)),
            ("ppi", (7, 8)),
            ("si", (8, 10)),
            ("ssi", (10, 12)),
        ];
        let rlfmi = RLFMIndexBackend::new(text, NoOpConverter::<u8>::default(), |_| ());

        let wrapper = SearchIndexWrapper::new(rlfmi);

        for (s, r) in ans {
            let search = wrapper.search(s);
            assert_eq!(search.get_range(), r);
        }
    }
    #[test]
    fn test_get_f() {
        let text = "mississippi\0".as_bytes();
        let mut ans = text.to_vec();
        ans.sort();
        let rlfmi = RLFMIndexBackend::new(text, NoOpConverter::<u8>::default(), |_| ());

        for (i, a) in ans.into_iter().enumerate() {
            let f = rlfmi.get_f(i);
            assert_eq!(f, a);
        }
    }

    #[test]
    fn test_fl_map() {
        let text = "mississippi\0".as_bytes();
        let rlfmi = RLFMIndexBackend::new(text, NoOpConverter::<u8>::default(), |_| ());
        let cases = vec![5, 0, 7, 10, 11, 4, 1, 6, 2, 3, 8, 9];
        for (i, expected) in cases.into_iter().enumerate() {
            let actual = rlfmi.fl_map(i).unwrap();
            assert_eq!(actual, expected);
        }
    }
}
