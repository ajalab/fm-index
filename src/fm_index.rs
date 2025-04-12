use crate::backend::{HasPosition, HeapSize, SearchIndexBackend};
use crate::character::Character;
use crate::error::Error;
use crate::suffix_array::sais;
use crate::suffix_array::sample::SOSampledSuffixArray;
use crate::text::Text;

use serde::{Deserialize, Serialize};
use vers_vecs::WaveletMatrix;

/// An FM-Index, a succinct full-text index.
#[derive(Serialize, Deserialize)]
pub struct FMIndexBackend<C, S> {
    bw: WaveletMatrix,
    cs: Vec<usize>,
    suffix_array: S,
    _c: std::marker::PhantomData<C>,
}

impl<C, S> FMIndexBackend<C, S>
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
        let cs = sais::get_bucket_start_pos(&sais::count_chars(text));
        let sa = sais::build_suffix_array(text)?;
        let bw = Self::wavelet_matrix(text, &sa);

        Ok(FMIndexBackend {
            cs,
            bw,
            suffix_array: get_sample(&sa),
            _c: std::marker::PhantomData::<C>,
        })
    }

    fn wavelet_matrix<T>(text: &Text<C, T>, sa: &[usize]) -> WaveletMatrix
    where
        T: AsRef<[C]>,
    {
        let n = text.text().len();
        let mut bw = vec![0u64; n];
        for i in 0..n {
            let k = sa[i];
            if k > 0 {
                bw[i] = text.text()[k - 1].into_u64();
            }
        }

        WaveletMatrix::from_slice(&bw, text.max_bits() as u16)
    }
}

impl<C> HeapSize for FMIndexBackend<C, ()>
where
    C: Character,
{
    fn heap_size(&self) -> usize {
        self.bw.heap_size() + self.cs.capacity() * std::mem::size_of::<u64>()
    }
}

impl<C> HeapSize for FMIndexBackend<C, SOSampledSuffixArray>
where
    C: Character,
{
    fn heap_size(&self) -> usize {
        self.bw.heap_size()
            + self.cs.capacity() * std::mem::size_of::<u64>()
            + self.suffix_array.size()
    }
}

impl<C, S> SearchIndexBackend for FMIndexBackend<C, S>
where
    C: Character,
{
    type C = C;

    fn len(&self) -> usize {
        self.bw.len()
    }

    fn get_l(&self, i: usize) -> Self::C {
        Self::C::from_u64(self.bw.get_u64_unchecked(i))
    }

    fn lf_map(&self, i: usize) -> usize {
        let c = self.get_l(i);
        let c_count = self.cs[c.into_usize()];
        let rank = self.bw.rank_u64_unchecked(i, c.into_u64());
        c_count + rank
    }

    fn lf_map2(&self, c: C, i: usize) -> usize {
        self.cs[c.into_usize()] + self.bw.rank_u64_unchecked(i, c.into_u64())
    }

    fn get_f(&self, i: usize) -> Self::C {
        // binary search to find c s.t. cs[c] <= i < cs[c+1]
        // <=> c is the greatest index s.t. cs[c] <= i
        // invariant: c exists in [s, e)
        let mut s = 0;
        let mut e = self.cs.len();
        while e - s > 1 {
            let m = s + (e - s) / 2;
            if self.cs[m] <= i {
                s = m;
            } else {
                e = m;
            }
        }
        C::from_usize(s)
    }

    fn fl_map(&self, i: usize) -> Option<usize> {
        let c = self.get_f(i);
        Some(
            self.bw
                .select_u64_unchecked(i - self.cs[c.into_usize()], c.into_u64()),
        )
    }
}

impl<C> HasPosition for FMIndexBackend<C, SOSampledSuffixArray>
where
    C: Character,
{
    fn get_sa(&self, mut i: usize) -> usize {
        let mut steps = 0;
        loop {
            match self.suffix_array.get(i) {
                Some(sa) => {
                    return (sa + steps) % self.bw.len();
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
    use crate::suffix_array::sample::SOSampledSuffixArray;

    #[test]
    fn test_lf_map() -> Result<(), Error> {
        let text = "mississippi\0".as_bytes();
        let ans = vec![1, 6, 7, 2, 8, 10, 3, 9, 11, 4, 5, 0];
        let fm_index =
            FMIndexBackend::new(&Text::new(text), |sa| SOSampledSuffixArray::sample(sa, 2))?;
        let mut i = 0;
        for a in ans {
            i = fm_index.lf_map(i);
            assert_eq!(i, a);
        }
        Ok(())
    }

    #[test]
    fn test_fl_map() -> Result<(), Error> {
        let text = "mississippi\0".as_bytes();
        let fm_index =
            FMIndexBackend::new(&Text::new(text), |sa| SOSampledSuffixArray::sample(sa, 2))?;
        let cases = vec![5usize, 0, 7, 10, 11, 4, 1, 6, 2, 3, 8, 9];
        for (i, expected) in cases.into_iter().enumerate() {
            let actual = fm_index.fl_map(i).unwrap();
            assert_eq!(actual, expected);
        }
        Ok(())
    }
}
