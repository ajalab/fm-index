use crate::backend::{HasPosition, HeapSize, SearchIndexBackend};
use crate::character::Character;
use crate::error::Error;
use crate::suffix_array::sais;
use crate::suffix_array::sample::SOSampledSuffixArray;
use crate::text::Text;

use serde::{Deserialize, Serialize};
use vers_vecs::{BitVec, RsVec, WaveletMatrix};

/// A Run-Length FM-index.
///
/// This can be more space-efficient than the FM-index, but is slower.
#[derive(Serialize, Deserialize)]
pub struct RLFMIndexBackend<C, S> {
    suffix_array: S,
    s: WaveletMatrix,
    b: RsVec,
    bp: RsVec,
    cs: Vec<usize>,
    len: usize,
    _c: std::marker::PhantomData<C>,
}

impl<C, S> RLFMIndexBackend<C, S>
where
    C: Character,
{
    pub(crate) fn new<T>(
        text: &Text<C, T>,
        get_sample: impl Fn(&[usize]) -> S,
    ) -> Result<Self, Error>
    where
        T: AsRef<[C]>,
    {
        let n = text.text().len();
        let m = text.max_character().into_usize() + 1;
        let sa = sais::build_suffix_array(text)?;

        let mut c0 = C::from_u64(0);
        // sequence of run heads
        let mut s = Vec::new();
        // sequence of run lengths
        // run length `l` is encoded as 10^{l-1}
        let mut b = BitVec::new();
        let mut runs_by_char: Vec<Vec<usize>> = vec![vec![]; m];
        for &k in &sa {
            let c = if k > 0 {
                text.text()[k - 1]
            } else {
                text.text()[n - 1]
            };
            // We do not allow consecutive occurrences of zeroes,
            // so text[sa[0] - 1] = text[n - 2] is not zero.
            if c0.into_u64() != c.into_u64() {
                s.push(c);
                b.append(true);
                runs_by_char[c.into_usize()].push(1);
            } else {
                b.append(false);
                match runs_by_char[c.into_usize()].last_mut() {
                    Some(r) => *r += 1,
                    None => unreachable!(),
                };
            }
            c0 = c;
        }
        let s: Vec<u64> = s.into_iter().map(|c| c.into_u64()).collect();
        let s = WaveletMatrix::from_slice(&s, text.max_bits() as u16);
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
        Ok(RLFMIndexBackend {
            suffix_array: get_sample(&sa),
            s,
            b,
            bp,
            cs,
            len: n,
            _c: std::marker::PhantomData::<C>,
        })
    }
}

impl<C> HeapSize for RLFMIndexBackend<C, ()>
where
    C: Character,
{
    fn heap_size(&self) -> usize {
        self.s.heap_size()
            + self.b.heap_size()
            + self.bp.heap_size()
            + self.cs.capacity() * std::mem::size_of::<u64>()
    }
}

impl<C> HeapSize for RLFMIndexBackend<C, SOSampledSuffixArray>
where
    C: Character,
{
    fn heap_size(&self) -> usize {
        self.s.heap_size()
            + self.b.heap_size()
            + self.bp.heap_size()
            + self.cs.capacity() * std::mem::size_of::<u64>()
            + self.suffix_array.size()
    }
}

impl<C, S> SearchIndexBackend for RLFMIndexBackend<C, S>
where
    C: Character,
{
    type C = C;

    fn len(&self) -> usize {
        self.len
    }

    fn get_l(&self, i: usize) -> C {
        // note: b[0] is always 1
        C::from_u64(self.s.get_u64_unchecked(self.b.rank1(i + 1) - 1))
    }

    fn lf_map(&self, i: usize) -> usize {
        let c = self.get_l(i);
        let j = self.b.rank1(i);
        let nr = self.s.rank_u64_unchecked(j, c.into_u64());

        self.bp.select1(self.cs[c.into_usize()] + nr) + i - self.b.select1(j)
    }

    fn lf_map2(&self, c: C, i: usize) -> usize {
        let j = self.b.rank1(i);
        let nr = self.s.rank_u64_unchecked(j, c.into_u64());
        if self.get_l(i).into_u64() != c.into_u64() {
            self.bp.select1(self.cs[c.into_usize()] + nr)
        } else {
            self.bp.select1(self.cs[c.into_usize()] + nr) + i - self.b.select1(j)
        }
    }

    fn get_f(&self, i: usize) -> Self::C {
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
        C::from_usize(s)
    }

    fn fl_map(&self, i: usize) -> Option<usize> {
        let c = self.get_f(i);
        let j = self.bp.rank1(i + 1) - 1;
        let p = self.bp.select1(j);
        let m = self
            .s
            .select_u64_unchecked(j - self.cs[c.into_usize()], c.into_u64());
        let n = self.b.select1(m);
        Some(n + i - p)
    }
}

impl<C> HasPosition for RLFMIndexBackend<C, SOSampledSuffixArray>
where
    C: Character,
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
    use crate::wrapper::SearchIndexWrapper;

    #[test]
    fn test_s() {
        let text = "mississippi\0".as_bytes();
        let rlfmi = RLFMIndexBackend::new(&Text::new(&text), |_| ()).unwrap();
        let ans = "ipsm\0pisi".as_bytes();
        for (i, a) in ans.iter().enumerate() {
            let l: u8 = rlfmi.s.get_u64_unchecked(i) as u8;
            assert_eq!(l, *a);
        }
    }

    #[test]
    fn test_b() {
        let text = "mississippi\0".as_bytes();
        let rlfmi = RLFMIndexBackend::new(&Text::new(&text), |_| ()).unwrap();
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
        let rlfmi = RLFMIndexBackend::new(&Text::new(&text), |_| ()).unwrap();
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
        let rlfmi = RLFMIndexBackend::new(&Text::new(&text), |_| ()).unwrap();
        let ans = vec![(b'\0', 0), (b'i', 1), (b'm', 4), (b'p', 5), (b's', 7)];
        for (c, a) in ans {
            assert_eq!(rlfmi.cs[c as usize], a);
        }
    }

    #[test]
    fn test_get_l() {
        let text = "mississippi\0".as_bytes();
        let rlfmi = RLFMIndexBackend::new(&Text::new(&text), |_| ()).unwrap();
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
        let rlfmi = RLFMIndexBackend::new(&Text::new(&text), |_| ()).unwrap();

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
        let rlfmi = RLFMIndexBackend::new(&Text::new(&text), |_| ()).unwrap();
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
        let rlfmi = RLFMIndexBackend::new(&Text::new(&text), |_| ()).unwrap();

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
        let rlfmi = RLFMIndexBackend::new(&Text::new(&text), |_| ()).unwrap();

        for (i, a) in ans.into_iter().enumerate() {
            let f = rlfmi.get_f(i);
            assert_eq!(f, a);
        }
    }

    #[test]
    fn test_fl_map() {
        let text = "mississippi\0".as_bytes();
        let rlfmi = RLFMIndexBackend::new(&Text::new(&text), |_| ()).unwrap();
        let cases = vec![5usize, 0, 7, 10, 11, 4, 1, 6, 2, 3, 8, 9];
        for (i, expected) in cases.into_iter().enumerate() {
            let actual = rlfmi.fl_map(i).unwrap();
            assert_eq!(actual, expected);
        }
    }
}
