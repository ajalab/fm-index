use serde::{Deserialize, Serialize};

use crate::fm_index::FMIndexBackend;
use crate::frontend::{Search, SearchIndex, SearchWithLocate};
use crate::search::Search as SearchBackend;
use crate::suffix_array::{self, SuffixOrderSampledArray};
use crate::SearchIndexWithLocate;
use crate::{character::Character, converter::Converter};

/// An FM-Index, a succinct full-text index.
///
/// The FM-Index is both a search index as well as compact
/// representation of the text, all within less space than the
/// original text.
#[derive(Serialize, Deserialize)]
pub struct FMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    backend: FMIndexBackend<T, C, S>,
}

impl<T, C> FMIndex<T, C, ()>
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
        Self {
            backend: FMIndexBackend::create(text, converter, |_| ()),
        }
    }

    /// The size on the heap of the FM-Index.
    ///
    /// No suffix array information is stored in this index.
    pub fn size(&self) -> usize {
        self.backend.size()
    }
}

impl<T, C> FMIndex<T, C, SuffixOrderSampledArray>
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
        Self {
            backend: FMIndexBackend::create(text, converter, |sa| suffix_array::sample(sa, level)),
        }
    }

    /// The size on the heap of the FM-Index.
    ///
    /// No suffix array information is stored in this index.
    pub fn size(&self) -> usize {
        self.backend.size()
    }
}

impl<T, C, S> FMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    /// Search for a pattern in the text.
    ///
    /// Return a [`Search`] object with information about the search
    /// result.
    pub fn search<K>(&self, pattern: K) -> FMIndexSearch<T, C, S>
    where
        K: AsRef<[T]>,
    {
        FMIndexSearch::new(self.backend.search(pattern))
    }

    /// The length of the text.
    pub fn len(&self) -> u64 {
        self.backend.len()
    }
}

impl<T, C, S> SearchIndex<T> for FMIndex<T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    #[allow(refining_impl_trait)]
    fn search<K>(&self, pattern: K) -> FMIndexSearch<T, C, S>
    where
        K: AsRef<[T]>,
    {
        FMIndex::search(self, pattern)
    }
}

impl<T, C> SearchIndexWithLocate<T> for FMIndex<T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    #[allow(refining_impl_trait)]
    fn search<K>(&self, pattern: K) -> FMIndexSearch<T, C, SuffixOrderSampledArray>
    where
        K: AsRef<[T]>,
    {
        FMIndex::search(self, pattern)
    }
}

pub struct FMIndexSearch<'a, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    search_backend: SearchBackend<'a, FMIndexBackend<T, C, S>>,
}

impl<'a, T, C, S> FMIndexSearch<'a, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    fn new(search_backend: SearchBackend<'a, FMIndexBackend<T, C, S>>) -> Self {
        FMIndexSearch { search_backend }
    }

    /// Search in the current search result, refining it.
    ///
    /// This adds a prefix `pattern` to the existing pattern, and
    /// looks for those expanded patterns in the text.
    pub fn search<K>(&self, pattern: K) -> Self
    where
        K: AsRef<[T]>,
    {
        let search_backend = self.search_backend.search(pattern);
        FMIndexSearch { search_backend }
    }

    /// Get the number of matches.
    pub fn count(&self) -> u64 {
        self.search_backend.count()
    }

    /// Get an iterator that goes backwards through the text, producing
    /// [`Character`].
    pub fn iter_backward(&self, i: u64) -> impl Iterator<Item = T> + '_ {
        self.search_backend.iter_backward(i)
    }

    /// Get an iterator that goes forwards through the text, producing
    /// [`Character`].
    pub fn iter_forward(&self, i: u64) -> impl Iterator<Item = T> + '_ {
        self.search_backend.iter_forward(i)
    }
}

impl<T, C, S> Search<T> for FMIndexSearch<'_, T, C, S>
where
    T: Character,
    C: Converter<T>,
{
    fn search<K>(&self, pattern: K) -> Self
    where
        K: AsRef<[T]>,
    {
        FMIndexSearch::search(self, pattern)
    }

    fn count(&self) -> u64 {
        FMIndexSearch::count(self)
    }

    fn iter_backward(&self, i: u64) -> impl Iterator<Item = T> {
        FMIndexSearch::iter_backward(self, i)
    }

    fn iter_forward(&self, i: u64) -> impl Iterator<Item = T> {
        FMIndexSearch::iter_forward(self, i)
    }
}

impl<T, C> FMIndexSearch<'_, T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    /// List the position of all occurrences.
    pub fn locate(&self) -> Vec<u64> {
        self.search_backend.locate()
    }
}

impl<T, C> SearchWithLocate<T> for FMIndexSearch<'_, T, C, SuffixOrderSampledArray>
where
    T: Character,
    C: Converter<T>,
{
    fn locate(&self) -> Vec<u64> {
        FMIndexSearch::locate(self)
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
