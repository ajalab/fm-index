use crate::fm_index::FMIndex as FMIndexBackend;
use crate::rlfmi::RLFMIndex as RLFMIndexBackend;
use crate::suffix_array::sample::SuffixOrderSampledArray;
use crate::wrapper::SearchWrapper;
use crate::{converter::Converter, wrapper::SearchIndexWrapper, Character};

pub trait SearchIndex<T> {
    fn search<K>(&self, pattern: K) -> impl Search<T>
    where
        K: AsRef<[T]>;

    fn len(&self) -> u64;
}

pub trait Search<'a, T> {
    fn search<K: AsRef<[T]>>(&self, pattern: K) -> Self;
    fn count(&self) -> u64;
    fn iter_backward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a;
    fn iter_forward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a;
}

trait SearchWithLocate<'a, T>: Search<'a, T> {
    fn locate(&self) -> Vec<u64>;
}

struct FMIndex<T: Character, C: Converter<T>>(SearchIndexWrapper<FMIndexBackend<T, C, ()>>);
struct FMIndexSearch<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, FMIndexBackend<T, C, ()>>,
);

pub struct FMIndexWithLocate<T: Character, C: Converter<T>>(
    SearchIndexWrapper<FMIndexBackend<T, C, SuffixOrderSampledArray>>,
);
pub struct FMIndexSearchWithLocate<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, FMIndexBackend<T, C, SuffixOrderSampledArray>>,
);

pub struct RLFMIndex<T: Character, C: Converter<T>>(SearchIndexWrapper<RLFMIndexBackend<T, C, ()>>);
pub struct RLFMIndexSearch<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, RLFMIndexBackend<T, C, ()>>,
);

pub struct RLFMIndexWithLocate<T: Character, C: Converter<T>>(
    SearchIndexWrapper<RLFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);
pub struct RLFMIndexSearchWithLocate<'a, T: Character, C: Converter<T>>(
    SearchWrapper<'a, RLFMIndexBackend<T, C, SuffixOrderSampledArray>>,
);

impl<T: Character, C: Converter<T>> FMIndex<T, C> {
    pub fn new(text: Vec<T>, converter: C) -> Self {
        FMIndex(SearchIndexWrapper::new(FMIndexBackend::count_only(
            text, converter,
        )))
    }
}

impl<T: Character, C: Converter<T>> FMIndexWithLocate<T, C> {
    pub fn new(text: Vec<T>, converter: C, level: usize) -> Self {
        FMIndexWithLocate(SearchIndexWrapper::new(FMIndexBackend::new(
            text, converter, level,
        )))
    }
}

impl<T: Character, C: Converter<T>> RLFMIndex<T, C> {
    pub fn new(text: Vec<T>, converter: C) -> Self {
        RLFMIndex(SearchIndexWrapper::new(RLFMIndexBackend::count_only(
            text, converter,
        )))
    }
}

impl<T: Character, C: Converter<T>> RLFMIndexWithLocate<T, C> {
    pub fn new(text: Vec<T>, converter: C, level: usize) -> Self {
        RLFMIndexWithLocate(SearchIndexWrapper::new(RLFMIndexBackend::new(
            text, converter, level,
        )))
    }
}

macro_rules! impl_search_index {
    ($t:ty, $s:ident) => {
        impl<T: Character, C: Converter<T>> SearchIndex<T> for $t {
            fn search<K>(&self, pattern: K) -> impl Search<T>
            where
                K: AsRef<[T]>,
            {
                $s(self.0.search(pattern))
            }

            fn len(&self) -> u64 {
                self.0.len()
            }
        }
        // inherent
        impl<T: Character, C: Converter<T>> $t {
            pub fn search<K>(&self, pattern: K) -> impl Search<T>
            where
                K: AsRef<[T]>,
            {
                SearchIndex::search(self, pattern)
            }
            pub fn len(&self) -> u64 {
                SearchIndex::len(self)
            }
        }
    };
}

macro_rules! impl_search {
    ($t:ty) => {
        impl<'a, T: Character, C: Converter<T>> Search<'a, T> for $t {
            fn search<K>(&self, pattern: K) -> Self
            where
                K: AsRef<[T]>,
            {
                Self(self.0.search(pattern))
            }

            fn count(&self) -> u64 {
                self.0.count()
            }

            fn iter_backward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a {
                self.0.iter_backward(i)
            }

            fn iter_forward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a {
                self.0.iter_forward(i)
            }
        }
        // inherent
        impl<'a, T: Character, C: Converter<T>> $t {
            pub fn search<K>(&self, pattern: K) -> Self
            where
                K: AsRef<[T]>,
            {
                Search::search(self, pattern)
            }

            pub fn count(&self) -> u64 {
                Search::count(self)
            }

            pub fn iter_backward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a {
                Search::iter_backward(self, i)
            }

            pub fn iter_forward(&'a self, i: u64) -> impl Iterator<Item = T> + 'a {
                Search::iter_forward(self, i)
            }
        }
    };
}

macro_rules! impl_search_locate {
    ($t:ty) => {
        impl<'a, T: Character, C: Converter<T>> SearchWithLocate<'a, T> for $t {
            fn locate(&self) -> Vec<u64> {
                self.0.locate()
            }
        }
        // inherent
        impl<'a, T: Character, C: Converter<T>> $t {
            pub fn locate(&self) -> Vec<u64> {
                SearchWithLocate::locate(self)
            }
        }
    };
}

impl_search_index!(FMIndex<T, C>, FMIndexSearch);
impl_search!(FMIndexSearch<'a, T, C>);
impl_search_index!(FMIndexWithLocate<T, C>, FMIndexSearchWithLocate);
impl_search!(FMIndexSearchWithLocate<'a, T, C>);
impl_search_locate!(FMIndexSearchWithLocate<'a, T, C>);
impl_search_index!(RLFMIndex<T, C>, RLFMIndexSearch);
impl_search!(RLFMIndexSearch<'a, T, C>);
impl_search_index!(RLFMIndexWithLocate<T, C>, RLFMIndexSearchWithLocate);
impl_search!(RLFMIndexSearchWithLocate<'a, T, C>);
impl_search_locate!(RLFMIndexSearchWithLocate<'a, T, C>);
