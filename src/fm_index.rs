use crate::character::{prepare_text, Character};
#[cfg(doc)]
use crate::converter;
use crate::converter::Converter;
use crate::iter::{FMIndexBackend, HasPosition};
use crate::suffix_array::sais;
use crate::suffix_array::sample::{self, SuffixOrderSampledArray};
use crate::{seal, HeapSize};
use crate::{util, Search};

use serde::{Deserialize, Serialize};
use vers_vecs::WaveletMatrix;

/// An FM-Index, a succinct full-text index.
///
/// The FM-Index is both a search index as well as compact
/// representation of the text, all within less space than the
/// original text.
#[derive(Serialize, Deserialize)]
pub struct FMIndex<T, C, S> {
    bw: WaveletMatrix,
    cs: Vec<u64>,
    converter: C,
    suffix_array: S,
    _t: std::marker::PhantomData<T>,
}

impl<T, C> FMIndex<T, C, ()>
where
    T: Character,
    C: Converter<T>,
{
    pub(crate) fn count_only(text: Vec<T>, converter: C) -> Self {
        Self::create(text, converter, |_| ())
    }
}
impl<T, C> FMIndex<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    pub(crate) fn new(text: Vec<T>, converter: C, level: usize) -> Self {
        Self::create(text, converter, |sa| sample::sample(sa, level))
    }
}

// TODO: Refactor types (Converter converts T -> u64)
impl<T, C, S> FMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    fn create(text: Vec<T>, converter: C, get_sample: impl Fn(&[u64]) -> S) -> Self {
        let text = prepare_text(text);
        let cs = sais::get_bucket_start_pos(&sais::count_chars(&text, &converter));
        let sa = sais::build_suffix_array(&text, &converter);
        let bw = Self::wavelet_matrix(text, &sa, &converter);

        FMIndex {
            cs,
            bw,
            converter,
            suffix_array: get_sample(&sa),
            _t: std::marker::PhantomData::<T>,
        }
    }

    fn wavelet_matrix(text: Vec<T>, sa: &[u64], converter: &C) -> WaveletMatrix {
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

impl<T, C> FMIndex<T, C, ()>
where
    T: Character,
    C: Converter<T>,
{
    /// The size on the heap of the FM-Index.
    ///
    /// No suffix array information is stored in this index.
    pub fn size(&self) -> usize {
        self.bw.heap_size() + self.cs.capacity() * std::mem::size_of::<u64>()
    }
}

impl<T, C> FMIndex<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    /// The size on the heap of the FM-Index.
    ///
    /// Sampled suffix array data is stored in this index.
    pub fn size(&self) -> usize {
        self.bw.heap_size()
            + self.cs.capacity() * std::mem::size_of::<u64>()
            + self.suffix_array.size()
    }
}

impl<T, C> HeapSize for FMIndex<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    fn size(&self) -> usize {
        FMIndex::<T, C, SuffixOrderSampledArray>::size(self)
    }
}

impl<T, C> HeapSize for FMIndex<T, C, ()>
where
    T: Character,
    C: Converter<T>,
{
    fn size(&self) -> usize {
        FMIndex::<T, C, ()>::size(self)
    }
}

impl<T, C, S> seal::Sealed for FMIndex<T, C, S> {}

impl<T, C, S> FMIndexBackend for FMIndex<T, C, S>
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
        let c_count = self.cs[c.into() as usize];
        let rank = self.bw.rank_u64_unchecked(i as usize, c.into()) as u64;
        c_count + rank
    }

    fn lf_map2(&self, c: T, i: u64) -> u64 {
        let c = self.converter.convert(c);
        self.cs[c.into() as usize] + self.bw.rank_u64_unchecked(i as usize, c.into()) as u64
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

    fn fl_map(&self, i: u64) -> u64 {
        let c = self.get_f(i);
        self.bw
            .select_u64_unchecked(i as usize - self.cs[c.into() as usize] as usize, c.into())
            as u64
    }

    fn fl_map2(&self, c: Self::T, i: u64) -> u64 {
        let c = self.converter.convert(c);
        self.bw
            .select_u64_unchecked((i - self.cs[c.into() as usize]) as usize, c.into())
            as u64
    }

    fn get_converter(&self) -> &Self::C {
        &self.converter
    }
}

impl<T, C> HasPosition for FMIndex<T, C, SuffixOrderSampledArray>
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::RangeConverter;

    #[test]
    fn test_small() {
        let text = "mississippi".to_string().into_bytes();
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

        let fm_index = FMIndex::new(text, RangeConverter::new(b'a', b'z'), 2);

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
        let fm_index = FMIndex::count_only(text, RangeConverter::new(b'a', b'z'));

        assert_eq!(fm_index.search("m").count(), 1);
        assert_eq!(fm_index.search("ssi").count(), 1);
        assert_eq!(fm_index.search("iss").count(), 2);
        assert_eq!(fm_index.search("p").count(), 2);
        assert_eq!(fm_index.search("\0").count(), 2);
        assert_eq!(fm_index.search("\0i").count(), 1);
    }

    #[test]
    fn test_utf8() {
        let text = "みんなみんなきれいだな"
            .chars()
            .map(|c| c as u32)
            .collect::<Vec<u32>>();
        let ans = vec![
            ("み", vec![0, 3]),
            ("みん", vec![0, 3]),
            ("な", vec![2, 5, 10]),
        ];
        let fm_index = FMIndex::new(text, RangeConverter::new('あ' as u32, 'ん' as u32), 2);

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
        let text = "mississippi".to_string().into_bytes();
        let ans = vec![1, 6, 7, 2, 8, 10, 3, 9, 11, 4, 5, 0];
        let fm_index = FMIndex::new(text, RangeConverter::new(b'a', b'z'), 2);
        let mut i = 0;
        for a in ans {
            i = fm_index.lf_map(i);
            assert_eq!(i, a);
        }
    }

    #[test]
    fn test_fl_map() {
        let text = "mississippi".to_string().into_bytes();
        let fm_index = FMIndex::new(text, RangeConverter::new(b'a', b'z'), 2);
        let cases = vec![5u64, 0, 7, 10, 11, 4, 1, 6, 2, 3, 8, 9];
        for (i, expected) in cases.into_iter().enumerate() {
            let actual = fm_index.fl_map(i as u64);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_search_backward() {
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string().into_bytes();
        let word_pairs = vec![("ipsum", " dolor"), ("sit", " amet"), ("sed", " do")];
        let fm_index = FMIndex::new(text, RangeConverter::new(b' ', b'~'), 2);
        for (fst, snd) in word_pairs {
            let search1 = fm_index.search(snd).search(fst);
            let concat = fst.to_owned() + snd;
            let search2 = fm_index.search(&concat);
            assert!(search1.count() > 0);
            assert_eq!(search1.count(), search2.count());
            assert_eq!(search1.locate(), search2.locate());
        }
    }

    #[test]
    fn test_iter_backward() {
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string().into_bytes();
        let index = FMIndex::count_only(text, RangeConverter::new(b' ', b'~'));
        let search = index.search("sit ");
        let mut prev_seq = search.iter_backward(0).take(6).collect::<Vec<_>>();
        prev_seq.reverse();
        assert_eq!(prev_seq, b"dolor ".to_owned());
    }

    #[test]
    fn test_iter_forward() {
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string().into_bytes();
        let index = FMIndex::count_only(text, RangeConverter::new(b' ', b'~'));
        let search = index.search("sit ");
        let next_seq = search.iter_forward(0).take(10).collect::<Vec<_>>();
        assert_eq!(next_seq, b"sit amet, ".to_owned());
    }
}
