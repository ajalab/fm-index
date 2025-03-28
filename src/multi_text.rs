use std::ops::{Rem, Sub};

use crate::backend::{HasMultiTexts, HasPosition, SearchIndexBackend};
use crate::character::{prepare_text, Character};
#[cfg(doc)]
use crate::converter;
use crate::converter::Converter;
use crate::suffix_array::sais;
use crate::suffix_array::sample::SuffixOrderSampledArray;
use crate::text::TextId;
use crate::util;
use crate::HeapSize;

use serde::{Deserialize, Serialize};
use vers_vecs::{BitVec, RsVec, WaveletMatrix};

// An FM-Index supporting multiple \0 separated texts
#[derive(Serialize, Deserialize)]
pub struct MultiTextFMIndexBackend<T, C, S> {
    bw: WaveletMatrix,
    cs: Vec<u64>,
    converter: C,
    suffix_array: S,
    doc: Vec<u64>,
    // The index of the first text in the suffix array
    sa_idx_first_text: u64,
    _t: std::marker::PhantomData<T>,
}

// TODO: Refactor types (Converter converts T -> u64)
impl<T, C, S> MultiTextFMIndexBackend<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    pub(crate) fn new(text: Vec<T>, converter: C, get_sample: impl Fn(&[u64]) -> S) -> Self {
        let text = prepare_text(text);
        let cs = sais::get_bucket_start_pos(&sais::count_chars(&text, &converter));
        let sa = sais::build_suffix_array(&text, &converter);
        let bw = Self::wavelet_matrix(&text, &sa, &converter);
        let (doc, sa_idx_first_text) = Self::doc(&text, &bw, &sa);

        MultiTextFMIndexBackend {
            cs,
            bw,
            converter,
            suffix_array: get_sample(&sa),
            doc,
            sa_idx_first_text,
            _t: std::marker::PhantomData::<T>,
        }
    }

    fn doc(text: &[T], bw: &WaveletMatrix, sa: &[u64]) -> (Vec<u64>, u64) {
        let mut end_marker_bits = BitVec::from_zeros(text.len());
        let mut end_marker_count = 0;
        for (i, c) in text.iter().enumerate() {
            if c.is_zero() {
                end_marker_bits.set(i, 1).unwrap();
                end_marker_count += 1;
            }
        }
        let end_marker_flags = RsVec::from_bit_vec(end_marker_bits);

        let mut end_marker_rank_l = 0;
        let mut doc = vec![0; end_marker_count];
        let mut sa_idx_first_text = 0;
        while let Some(p) = bw.select_u64(end_marker_rank_l, 0) {
            let end_marker_idx = modular_sub(sa[p] as usize, 1, sa.len());
            let text_id = end_marker_flags.rank1(end_marker_idx) as u64;
            if text_id == (end_marker_count as u64 - 1) {
                sa_idx_first_text = p;
            }
            doc[end_marker_rank_l] = text_id;

            end_marker_rank_l += 1;
        }

        (doc, sa_idx_first_text as u64)
    }

    fn wavelet_matrix(text: &[T], sa: &[u64], converter: &C) -> WaveletMatrix {
        let n = text.len();
        let mut bw = vec![T::zero(); n];
        for i in 0..n {
            let k = sa[i] as usize;
            if k > 0 {
                bw[i] = converter.convert(text[k - 1]);
            }
        }
        let bw = bw.into_iter().map(|c| c.into()).collect::<Vec<u64>>();

        WaveletMatrix::from_slice(&bw, (util::log2(converter.len() - 1) + 1) as u16)
    }
}

impl<T, C> HeapSize for MultiTextFMIndexBackend<T, C, ()>
where
    T: Character,
    C: Converter<T>,
{
    fn heap_size(&self) -> usize {
        self.bw.heap_size() + self.cs.capacity() * std::mem::size_of::<u64>()
    }
}

impl<T, C> HeapSize for MultiTextFMIndexBackend<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    fn heap_size(&self) -> usize {
        self.bw.heap_size()
            + self.cs.capacity() * std::mem::size_of::<u64>()
            + self.suffix_array.size()
            + self.doc.capacity() * std::mem::size_of::<usize>()
    }
}

impl<T, C, S> SearchIndexBackend for MultiTextFMIndexBackend<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    type T = T;
    type C = C;

    fn len(&self) -> u64 {
        self.bw.len() as u64
    }

    fn get_l(&self, i: u64) -> Self::T {
        Self::T::from_u64(self.bw.get_u64_unchecked(i as usize))
    }

    fn lf_map(&self, i: u64) -> u64 {
        let c = self.get_l(i);
        let rank = self.bw.rank_u64_unchecked(i as usize, c.into());
        if c.is_zero() {
            match i.cmp(&self.sa_idx_first_text) {
                std::cmp::Ordering::Less => rank as u64 + 1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => rank as u64,
            }
        } else {
            let c_count = self.cs[c.into() as usize];
            rank as u64 + c_count
        }
    }

    fn lf_map2(&self, c: T, i: u64) -> u64 {
        let c = self.converter.convert(c);
        let rank = self.bw.rank_u64_unchecked(i as usize, c.into());
        if c.is_zero() {
            match i.cmp(&self.sa_idx_first_text) {
                std::cmp::Ordering::Less => rank as u64 + 1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => rank as u64,
            }
        } else {
            let c_count = self.cs[c.into() as usize];
            rank as u64 + c_count
        }
    }

    fn get_f(&self, i: u64) -> Self::T {
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
        T::from_u64(s as u64)
    }

    fn fl_map(&self, i: u64) -> Option<u64> {
        let c = self.get_f(i);
        if c.is_zero() {
            None
        } else {
            Some(
                self.bw.select_u64_unchecked(
                    i as usize - self.cs[c.into() as usize] as usize,
                    c.into(),
                ) as u64,
            )
        }
    }

    fn get_converter(&self) -> &Self::C {
        &self.converter
    }
}

impl<T, C> HasPosition for MultiTextFMIndexBackend<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    fn get_sa(&self, mut i: u64) -> u64 {
        let mut steps = 0;
        loop {
            match self.suffix_array.get(i) {
                Some(sa) => {
                    return (sa + steps) % self.bw.len() as u64;
                }
                None => {
                    i = self.lf_map(i);
                    steps += 1;
                }
            }
        }
    }
}

impl<T, C, S> HasMultiTexts for MultiTextFMIndexBackend<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    fn text_id(&self, mut i: u64) -> TextId {
        loop {
            if self.get_l(i).is_zero() {
                let text_id_prev = self.doc[self.bw.rank_u64_unchecked(i as usize, 0)];
                let text_id = modular_add(text_id_prev, 1, self.doc.len() as u64);
                return TextId::from(text_id);
            } else {
                i = self.lf_map(i);
            }
        }
    }
}

fn modular_add<T: Rem<Output = T> + Ord + num_traits::Zero>(a: T, b: T, m: T) -> T {
    debug_assert!(T::zero() <= a && a <= m);
    debug_assert!(T::zero() <= b && b <= m);

    (a + b) % m
}

fn modular_sub<T: Sub<Output = T> + Ord + num_traits::Zero>(a: T, b: T, m: T) -> T {
    debug_assert!(T::zero() <= a && a <= m);
    debug_assert!(T::zero() <= b && b <= m);

    if a >= b {
        a - b
    } else {
        m + a - b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil;
    use crate::{converter::IdConverter, suffix_array::sample};
    use rand::{rngs::StdRng, Rng, SeedableRng};

    #[test]
    fn test_lf_map_random() {
        let text_size = 512;
        let attempts = 100;
        let alphabet_size = 8;
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..attempts {
            let text = testutil::build_text(|| rng.gen::<u8>() % alphabet_size, text_size);
            let converter = IdConverter::new::<u8>();
            let suffix_array = testutil::build_suffix_array(&text);
            let inv_suffix_array = testutil::build_inv_suffix_array(&suffix_array);
            let fm_index =
                MultiTextFMIndexBackend::new(text.clone(), converter, |sa| sample::sample(sa, 0));

            let mut lf_map_expected = vec![0; text_size];
            let mut lf_map_actual = vec![0; text_size];
            for i in 0..text_size {
                let k = modular_sub(suffix_array[i] as usize, 1, text_size);
                lf_map_expected[i] = inv_suffix_array[k];
                lf_map_actual[i] = fm_index.lf_map(i as u64);
            }

            assert_eq!(lf_map_expected, lf_map_actual);
        }
    }

    #[test]
    fn test_get_text_id() {
        let text = "foo\0bar\0baz\0".as_bytes();
        let converter = IdConverter::new::<u8>();
        let suffix_array = testutil::build_suffix_array(text);
        let fm_index =
            MultiTextFMIndexBackend::new(text.to_vec(), converter, |sa| sample::sample(sa, 0));

        for (i, &char_pos) in suffix_array.iter().enumerate() {
            let text_id_expected = TextId::from(
                text[..(char_pos as usize)]
                    .iter()
                    .filter(|&&c| c == 0)
                    .count() as u64,
            );
            let text_id_actual = fm_index.text_id(i as u64);
            assert_eq!(
                text_id_expected, text_id_actual,
                "the text ID of a character at position {} ({} in suffix array) must be {:?}",
                char_pos, i, text_id_expected
            );
        }
    }

    #[test]
    fn test_get_text_id_random() {
        let text_size = 512;
        let attempts = 100;
        let alphabet_size = 8;
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..attempts {
            let text = testutil::build_text(|| rng.gen::<u8>() % alphabet_size, text_size);
            let converter = IdConverter::new::<u8>();
            let suffix_array = testutil::build_suffix_array(&text);
            let fm_index =
                MultiTextFMIndexBackend::new(text.clone(), converter, |sa| sample::sample(sa, 0));

            for (i, &char_pos) in suffix_array.iter().enumerate() {
                let text_id_expected = TextId::from(
                    text[..(char_pos as usize)]
                        .iter()
                        .filter(|&&c| c == 0)
                        .count() as u64,
                );
                let text_id_actual = fm_index.text_id(i as u64);
                assert_eq!(
                    text_id_expected, text_id_actual,
                    "the text ID of a character at position {} ({} in suffix array) must be {:?}. text={:?}, suffix_array={:?}",
                    char_pos, i, text_id_expected, text, suffix_array,
                );
            }
        }
    }
}
