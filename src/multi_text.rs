use std::ops::Sub;

use crate::character::{prepare_text, Character};
#[cfg(doc)]
use crate::converter;
use crate::converter::{Converter, IndexWithConverter};
use crate::iter::{FMIndexBackend, HasPosition};
use crate::seal;
use crate::suffix_array::sais;
use crate::suffix_array::sample::{self, SuffixOrderSampledArray};
use crate::{util, Search};

use serde::{Deserialize, Serialize};
use vers_vecs::{BitVec, RsVec, WaveletMatrix};

/// An FM-Index, a succinct full-text index.
///
/// The FM-Index is both a search index as well as compact
/// representation of the text, all within less space than the
/// original text.
#[derive(Serialize, Deserialize)]
pub struct MultiTextFMIndex<T, C, S> {
    bw: WaveletMatrix,
    cs: Vec<u64>,
    converter: C,
    suffix_array: S,
    doc: Vec<usize>,
    _t: std::marker::PhantomData<T>,
}

impl<T, C> MultiTextFMIndex<T, C, ()>
where
    T: Character,
    C: Converter<T>,
{
    /// Create a new FM-Index from a text. The index only supports the count
    /// operation.
    ///
    /// - `text` is a vector of [`Character`]s.
    ///
    /// - `converter` is a [`Converter`] is used to convert the characters to a
    ///   smaller alphabet. Use [`converter::IdConverter`] if you don't need to
    ///   restrict the alphabet. Use [`converter::RangeConverter`] if you can
    ///   contrain characters to a particular range. See [`converter`] for more
    ///   details.
    pub fn count_only(text: Vec<T>, converter: C) -> Self {
        Self::create(text, converter, |_| ())
    }
}
impl<T, C> MultiTextFMIndex<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    /// Create a new FM-Index from a text. The index supports both the count
    /// and locate operations.
    ///
    /// - `text` is a vector of [`Character`]s.
    ///
    /// - `converter` is a [`Converter`] is used to convert the characters to a
    ///   smaller alphabet. Use [`converter::IdConverter`] if you don't need to
    ///   restrict the alphabet. Use [`converter::RangeConverter`] if you can
    ///   contrain characters to a particular range. See [`converter`] for more
    ///   details.
    ///
    /// - `level` is the sampling level to use for position lookup. A sampling
    ///   level of 0 means the most memory is used (a full suffix-array is
    ///   retained), while looking up positions is faster. A sampling level of
    ///   1 means half the memory is used, but looking up positions is slower.
    ///   Each increase in level halves the memory usage but slows down
    ///   position lookup.
    pub fn new(text: Vec<T>, converter: C, level: usize) -> Self {
        Self::create(text, converter, |sa| sample::sample(sa, level))
    }
}

// TODO: Refactor types (Converter converts T -> u64)
impl<T, C, S> MultiTextFMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    fn create(text: Vec<T>, converter: C, get_sample: impl Fn(&[u64]) -> S) -> Self {
        let text = prepare_text(text);
        let cs = sais::get_bucket_start_pos(&sais::count_chars(&text, &converter));
        let sa = Self::suffix_array(&text, &converter);
        let bw = Self::wavelet_matrix(&text, &sa, &converter);
        let doc = Self::doc(&text, &bw, &sa);

        MultiTextFMIndex {
            cs,
            bw,
            converter,
            suffix_array: get_sample(&sa),
            doc,
            _t: std::marker::PhantomData::<T>,
        }
    }

    /**
     * Compute the suffix array of the given text.
     * This algorithm is aware of the order of end markers (zeros).
     *
     * TODO: Integrate it to SA-IS algorithm.
     */
    fn suffix_array<K>(text: K, converter: &C) -> Vec<u64>
    where
        K: AsRef<[T]>,
    {
        let text = text.as_ref();
        let suffixes = (0..text.len())
            .map(|i| {
                text[i..]
                    .iter()
                    .enumerate()
                    .map(|(j, c)| {
                        let c = converter.convert(*c);
                        if c.is_zero() {
                            (c, i + j)
                        } else {
                            (c, 0)
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut sa = (0..text.len() as u64).collect::<Vec<_>>();
        sa.sort_by(|i, j| suffixes[*i as usize].cmp(&suffixes[*j as usize]));
        sa
    }

    fn doc(text: &[T], bw: &WaveletMatrix, sa: &[u64]) -> Vec<usize> {
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
        while let Some(p) = bw.select_u64(end_marker_rank_l, 0) {
            let end_marker_idx = modular_sub(sa[p] as usize, 1, sa.len());
            let text_id = end_marker_flags.rank1(end_marker_idx);
            doc[end_marker_rank_l] = text_id;

            end_marker_rank_l += 1;
        }

        doc
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

    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    pub fn search<K>(&self, pattern: K) -> Search<Self>
    where
        K: AsRef<[T]>,
    {
        Search::new(self).search(pattern)
    }

    /// The length of the text.
    pub fn len(&self) -> u64 {
        self.bw.len() as u64
    }
}

impl<T, C> MultiTextFMIndex<T, C, ()> {
    /// The size on the heap of the FM-Index.
    ///
    /// No suffix array information is stored in this index.
    pub fn size(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.bw.heap_size()
            + self.cs.len() * std::mem::size_of::<Vec<u64>>()
    }
}

impl<T, C> MultiTextFMIndex<T, C, SuffixOrderSampledArray> {
    /// The size on the heap of the FM-Index.
    ///
    /// Sampled suffix array data is stored in this index.
    pub fn size(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.bw.heap_size()
            + self.cs.len() * std::mem::size_of::<Vec<u64>>()
            + self.suffix_array.size()
    }
}

impl<T, C, S> seal::Sealed for MultiTextFMIndex<T, C, S> {}

impl<T, C, S> FMIndexBackend for MultiTextFMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    type T = T;

    fn len(&self) -> u64 {
        self.bw.len() as u64
    }

    fn get_l<L: seal::IsLocal>(&self, i: u64) -> Self::T {
        Self::T::from_u64(self.bw.get_u64_unchecked(i as usize))
    }

    fn lf_map<L: seal::IsLocal>(&self, i: u64) -> u64 {
        let c = self.get_l::<L>(i);
        let rank = self.bw.rank_u64_unchecked(i as usize, c.into());

        if c.is_zero() {
            self.doc[rank] as u64
        } else {
            let c_count = self.cs[c.into() as usize];
            rank as u64 + c_count
        }
    }

    fn lf_map2<L: seal::IsLocal>(&self, c: T, i: u64) -> u64 {
        let c = self.converter.convert(c);
        let rank = self.bw.rank_u64_unchecked(i as usize, c.into());

        if c.is_zero() {
            self.doc[rank] as u64
        } else {
            let c_count = self.cs[c.into() as usize];
            rank as u64 + c_count
        }
    }

    fn get_f<L: seal::IsLocal>(&self, i: u64) -> Self::T {
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

    fn fl_map<L: seal::IsLocal>(&self, _i: u64) -> u64 {
        todo!("implement inverse LF-mapping");
    }

    fn fl_map2<L: seal::IsLocal>(&self, _c: Self::T, _i: u64) -> u64 {
        todo!("implement inverse LF-mapping");
    }
}

impl<T, C> HasPosition for MultiTextFMIndex<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    fn get_sa<L: seal::IsLocal>(&self, mut i: u64) -> u64 {
        let mut steps = 0;
        loop {
            match self.suffix_array.get(i) {
                Some(sa) => {
                    return (sa + steps) % self.bw.len() as u64;
                }
                None => {
                    i = self.lf_map::<seal::Local>(i);
                    steps += 1;
                }
            }
        }
    }
}

impl<T, C, S> IndexWithConverter<T> for MultiTextFMIndex<T, C, S>
where
    C: Converter<T>,
    T: Character,
{
    type C = C;

    fn get_converter(&self) -> &Self::C {
        &self.converter
    }
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
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use super::*;
    use crate::{converter::IdConverter, seal::Local};

    #[test]
    fn test_lf_map() {
        let text_size = 4096;
        let text = generate_text_random(text_size, 8);

        let converter = IdConverter::new::<u8>();
        let suffix_array = MultiTextFMIndex::<_, _, ()>::suffix_array(&text, &converter);
        let inv_suffix_array = inv_suffix_array(&suffix_array);
        let fm_index = MultiTextFMIndex::new(text, converter, 0);

        let mut lf_map_expected = vec![0; text_size];
        let mut lf_map_actual = vec![0; text_size];
        for i in 0..text_size {
            lf_map_expected[i] =
                inv_suffix_array[modular_sub(suffix_array[i] as usize, 1, text_size)];
            lf_map_actual[i] = fm_index.lf_map::<Local>(i as u64);
        }

        assert_eq!(lf_map_expected, lf_map_actual);
    }

    #[test]
    fn test_search_locate() {
        let text_size = 1024;
        let alphabet_size = 8;
        let pattern_size_max = 128;
        let text = generate_text_random(text_size, alphabet_size);

        let fm_index = MultiTextFMIndex::new(text.clone(), IdConverter::new::<u8>(), 0);

        let mut rng = StdRng::seed_from_u64(0);
        for i in 0..1000 {
            let pattern_size = rng.gen::<usize>() % (pattern_size_max - 1) + 1;
            let pattern = (0..pattern_size)
                .map(|_| rng.gen::<u8>() % (alphabet_size - 1) + 1)
                .collect::<Vec<_>>();

            let mut positions_expected = Vec::new();
            for i in 0..=(text_size - pattern_size) {
                if text[i..i + pattern_size] == pattern {
                    positions_expected.push(i as u64);
                }
            }
            let mut positions_actual = fm_index.search(&pattern).locate();
            positions_actual.sort();

            assert_eq!(
                positions_expected, positions_actual,
                "i = {:?}, text = {:?}, pattern = {:?}",
                i, text, pattern
            );
        }
    }

    fn generate_text_random(text_size: usize, alphabet_size: u8) -> Vec<u8> {
        let mut rng = StdRng::seed_from_u64(0);

        (0..text_size)
            .map(|_| rng.gen::<u8>() % alphabet_size)
            .collect::<Vec<_>>()
    }

    fn inv_suffix_array(suffix_array: &[u64]) -> Vec<u64> {
        let mut isa = vec![0; suffix_array.len()];
        for (p, &i) in suffix_array.iter().enumerate() {
            isa[i as usize] = p as u64;
        }
        isa
    }
}
