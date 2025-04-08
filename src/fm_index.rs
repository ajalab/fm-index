use crate::backend::{HasPosition, HeapSize, SearchIndexBackend};
#[cfg(doc)]
use crate::converter;
use crate::converter::Converter;
use crate::suffix_array::sais;
use crate::suffix_array::sample::SuffixOrderSampledArray;
use crate::util;

use serde::{Deserialize, Serialize};
use vers_vecs::WaveletMatrix;

/// An FM-Index, a succinct full-text index.
#[derive(Serialize, Deserialize)]
pub struct FMIndexBackend<T, C, S> {
    bw: WaveletMatrix,
    cs: Vec<usize>,
    converter: C,
    suffix_array: S,
    _t: std::marker::PhantomData<T>,
}

impl<T, C, S> FMIndexBackend<T, C, S>
where
    T: Copy + Clone,
    C: Converter<Char = T>,
{
    pub(crate) fn new(text: &[T], converter: C, get_sample: impl Fn(&[usize]) -> S) -> Self {
        let cs = sais::get_bucket_start_pos(&sais::count_chars(text, &converter));
        let sa = sais::build_suffix_array(text, &converter);
        let bw = Self::wavelet_matrix(text, &sa, &converter);

        FMIndexBackend {
            cs,
            bw,
            converter,
            suffix_array: get_sample(&sa),
            _t: std::marker::PhantomData::<T>,
        }
    }

    fn wavelet_matrix(text: &[T], sa: &[usize], converter: &C) -> WaveletMatrix {
        let n = text.len();
        let mut bw = vec![0; n];
        for i in 0..n {
            let k = sa[i];
            bw[i] = converter.to_u64(text[util::modular_sub(k, 1, n)]);
        }

        WaveletMatrix::from_slice(
            &bw,
            (util::log2(converter.to_u64(converter.max_value())) + 1) as u16,
        )
    }
}

impl<T, C> HeapSize for FMIndexBackend<T, C, ()>
where
    T: Copy + Clone,
    C: Converter<Char = T>,
{
    fn heap_size(&self) -> usize {
        self.bw.heap_size() + self.cs.capacity() * std::mem::size_of::<usize>()
    }
}

impl<T, C> HeapSize for FMIndexBackend<T, C, SuffixOrderSampledArray>
where
    T: Copy + Clone,
    C: Converter<Char = T>,
{
    fn heap_size(&self) -> usize {
        self.bw.heap_size()
            + self.cs.capacity() * std::mem::size_of::<usize>()
            + self.suffix_array.size()
    }
}

impl<T, C, S> SearchIndexBackend for FMIndexBackend<T, C, S>
where
    T: Copy + Clone,
    C: Converter<Char = T>,
{
    type T = T;
    type C = C;

    fn len(&self) -> usize {
        self.bw.len()
    }

    fn get_l(&self, i: usize) -> Self::T {
        self.converter.from_u64(self.bw.get_u64_unchecked(i))
    }

    fn lf_map(&self, i: usize) -> usize {
        let c = self.get_l(i);
        let c_count = self.cs[self.converter.to_usize(c)];
        let rank = self
            .bw
            .rank_u64_unchecked(i, self.converter.to_u64(c));
        c_count + rank
    }

    fn lf_map2(&self, c: T, i: usize) -> usize {
        self.cs[self.converter.to_usize(c)]
            + self
                .bw
                .rank_u64_unchecked(i, self.converter.to_u64(c))
    }

    fn get_f(&self, i: usize) -> Self::T {
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
        self.converter.from_usize(s)
    }

    fn fl_map(&self, i: usize) -> Option<usize> {
        let c = self.get_f(i);
        Some(self.bw.select_u64_unchecked(
            i - self.cs[self.converter.to_usize(c)],
            self.converter.to_u64(c),
        ))
    }

    fn get_converter(&self) -> &Self::C {
        &self.converter
    }
}

impl<T, C> HasPosition for FMIndexBackend<T, C, SuffixOrderSampledArray>
where
    T: Copy + Clone,
    C: Converter<Char = T>,
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
    use crate::testutil;
    use crate::{converter::DefaultConverter, suffix_array::sample};
    use rand::{rngs::StdRng, Rng, SeedableRng};

    #[test]
    fn test_lf_map_random() {
        let text_size = 10;
        let attempts = 100;
        let alphabet_size = 8;
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..attempts {
            let text = testutil::build_text(|| rng.gen::<u8>() % alphabet_size + 1, text_size);
            let converter = DefaultConverter::<u8>::default();
            let suffix_array = testutil::build_suffix_array(&text);
            let inv_suffix_array = testutil::build_inv_suffix_array(&suffix_array);
            let fm_index = FMIndexBackend::new(&text, converter, |sa| sample::sample(sa, 0));

            let mut lf_map_expected = vec![0; text_size];
            let mut lf_map_actual = vec![0; text_size];
            for i in 0..text_size {
                let k = util::modular_sub(suffix_array[i] as usize, 1, text_size);
                lf_map_expected[i] = inv_suffix_array[k];
                lf_map_actual[i] = fm_index.lf_map(i);
            }

            assert_eq!(lf_map_expected, lf_map_actual, "text = {:?}", text);
        }
    }
}
