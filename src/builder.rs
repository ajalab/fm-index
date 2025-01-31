#[cfg(doc)]
use crate::converter;

use crate::{
    converter::{Converter, IdConverter},
    fm_index::{
        FMIndexCountOnlySearchIndex, FMIndexCountOnlySearchResult, FMIndexLocateSearchIndex,
        FMIndexLocateSearchResult,
    },
    rlfmi::{
        RLFMIndexCountOnlySearchIndex, RLFMIndexCountOnlySearchResult, RLFMIndexLocateSearchIndex,
        RLFMIndexLocateSearchResult,
    },
    suffix_array::SuffixOrderSampledArray,
    Character, FMIndex, RLFMIndex, SearchIndex, SearchIndexWithLocate, SearchResult,
};

enum SearchIndexWrapper<T: Character, C: Converter<T>> {
    FMIndexCountOnly(FMIndexCountOnlySearchIndex<T, C>),
    FMIndexLocate(FMIndexLocateSearchIndex<T, C>),
    RLFMIndexCountOnly(RLFMIndexCountOnlySearchIndex<T, C>),
    RLFMIndexLocate(RLFMIndexLocateSearchIndex<T, C>),
}

enum SearchIndexResultWrapper<T: Character> {
    FMIndexCountOnly(FMIndexCountOnlySearchIndex<T, IdConverter>),
    FMIndexLocate(FMIndexLocateSearchIndex<T, IdConverter>),
    RLFMIndexCountOnly(RLFMIndexCountOnlySearchIndex<T, IdConverter>),
    RLFMIndexLocate(RLFMIndexLocateSearchIndex<T, IdConverter>),
}

enum SearchResultWrapper<'a, T: Character, C: Converter<T>> {
    FMIndexCountOnly(FMIndexCountOnlySearchResult<'a, T, C>),
    FMIndexLocate(FMIndexLocateSearchResult<'a, T, C>),
    RLFMIndexCountOnly(RLFMIndexCountOnlySearchResult<'a, T, C>),
    RLFMIndexLocate(RLFMIndexLocateSearchResult<'a, T, C>),
}

impl<'a, T: Character, C: Converter<T>> SearchResult<'a, T> for SearchResultWrapper<'a, T, C> {
    fn search<K: AsRef<[T]>>(&self, pattern: K) -> Self {
        match self {
            SearchResultWrapper::FMIndexCountOnly(result) => {
                SearchResultWrapper::FMIndexCountOnly(result.search(pattern))
            }
            SearchResultWrapper::FMIndexLocate(result) => {
                SearchResultWrapper::FMIndexLocate(result.search(pattern))
            }
            SearchResultWrapper::RLFMIndexCountOnly(result) => {
                SearchResultWrapper::RLFMIndexCountOnly(result.search(pattern))
            }
            SearchResultWrapper::RLFMIndexLocate(result) => {
                SearchResultWrapper::RLFMIndexLocate(result.search(pattern))
            }
        }
    }

    fn count(&self) -> u64 {
        match self {
            SearchResultWrapper::FMIndexCountOnly(result) => result.count(),
            SearchResultWrapper::FMIndexLocate(result) => result.count(),
            SearchResultWrapper::RLFMIndexCountOnly(result) => result.count(),
            SearchResultWrapper::RLFMIndexLocate(result) => result.count(),
        }
    }

    fn iter_backward(&self, i: u64) -> Box<dyn Iterator<Item = T> + 'a> {
        match self {
            SearchResultWrapper::FMIndexCountOnly(result) => Box::new(result.iter_backward(i)),
            SearchResultWrapper::FMIndexLocate(result) => Box::new(result.iter_backward(i)),
            SearchResultWrapper::RLFMIndexCountOnly(result) => Box::new(result.iter_backward(i)),
            SearchResultWrapper::RLFMIndexLocate(result) => Box::new(result.iter_backward(i)),
        }
    }

    fn iter_forward(&self, i: u64) -> Box<dyn Iterator<Item = T> + 'a> {
        match self {
            SearchResultWrapper::FMIndexCountOnly(result) => Box::new(result.iter_forward(i)),
            SearchResultWrapper::FMIndexLocate(result) => Box::new(result.iter_forward(i)),
            SearchResultWrapper::RLFMIndexCountOnly(result) => Box::new(result.iter_forward(i)),
            SearchResultWrapper::RLFMIndexLocate(result) => Box::new(result.iter_forward(i)),
        }
    }
}

impl<T: Character, C: Converter<T>> SearchIndex<T> for SearchIndexWrapper<T, C> {
    type SearchResult<'a>
        = SearchResultWrapper<'a, T, C>
    where
        T: 'a,
        C: 'a;

    fn search<K: AsRef<[T]>>(&self, pattern: K) -> Self::SearchResult<'_> {
        match self {
            SearchIndexWrapper::FMIndexCountOnly(index) => {
                SearchResultWrapper::FMIndexCountOnly(index.search(pattern))
            }
            SearchIndexWrapper::FMIndexLocate(index) => {
                SearchResultWrapper::FMIndexLocate(index.search(pattern))
            }
            SearchIndexWrapper::RLFMIndexCountOnly(index) => {
                SearchResultWrapper::RLFMIndexCountOnly(index.search(pattern))
            }
            SearchIndexWrapper::RLFMIndexLocate(index) => {
                SearchResultWrapper::RLFMIndexLocate(index.search(pattern))
            }
        }
    }

    fn len(&self) -> u64 {
        match self {
            SearchIndexWrapper::FMIndexCountOnly(index) => index.len(),
            SearchIndexWrapper::FMIndexLocate(index) => index.len(),
            SearchIndexWrapper::RLFMIndexCountOnly(index) => index.len(),
            SearchIndexWrapper::RLFMIndexLocate(index) => index.len(),
        }
    }
}

/// Construct a search index
///
/// If you don't configure anything before building, a search index is created
/// that offers all capabilities and maximum performance, at the cost of higher
/// memory usage.
///
/// If you know that your characters fit in a particular range, you can reduce
/// memory usage by passing in a `converter::RangeConverter`, using the
/// `with_converter` method.
///
/// If you want to reduce memory usage you can use the `sampling_level` method,
/// which trades a slower `locate` method for less memory usage. In addition
/// you can also `run_length_encoding`, which uses a different index that is
/// slower but uses less memory.
///
/// If you only need count queries you can further reduce memory usage by using
/// `count_only`. This disables the `locate` feature altogether so you cannot
/// use `sampling_level` combined with this.
///
/// Default behavior: all capabilities with maximum performance but most memory
/// usage.
/// ```rust
/// use fm_index::SearchIndexBuilder;
/// let builder = SearchIndexBuilder::new();
/// let index = builder.build("text".as_bytes().to_vec());
/// ```
///
/// Custom converter with a smaller range means less memory usage:
/// ```rust
/// use fm_index::SearchIndexBuilder;
/// use fm_index::converter::RangeConverter;
/// let converter = RangeConverter::new(b' ', b'~');
/// let builder = SearchIndexBuilder::with_converter(converter);
/// let index = builder.build("text".as_bytes().to_vec());
/// ```
///
/// Sampling level, smaller but slower locate:
/// ```rust
/// use fm_index::SearchIndexBuilder;
/// let builder = SearchIndexBuilder::new().sampling_level(2);
/// let index = builder.build("text".as_bytes().to_vec());
/// ```
///
/// Count only, smaller at the cost of locate.
/// ```rust
/// use fm_index::SearchIndexBuilder;
/// let builder = SearchIndexBuilder::new().count_only();
/// let index = builder.build("text".as_bytes().to_vec());
/// ```
///
/// Run-length encoding, smaller but slower.
/// ```rust
/// use fm_index::SearchIndexBuilder;
/// let builder = SearchIndexBuilder::new().run_length_encoding();
/// let index = builder.build("text".as_bytes().to_vec());
/// ```
///
/// Almost the smallest index (a range converter can still shrink it further):
/// ```rust
/// use fm_index::SearchIndexBuilder;
/// let builder = SearchIndexBuilder::new().count_only().run_length_encoding();
/// let index = builder.build("text".as_bytes().to_vec());
/// ```
pub struct SearchIndexBuilder<T, C>
where
    T: Character,
    C: Converter<T>,
{
    converter: C,
    sampling_level: Option<usize>,
    _t: std::marker::PhantomData<T>,
}

impl<T> SearchIndexBuilder<T, IdConverter>
where
    T: Character,
{
    /// Construct a new search index builder
    ///
    /// This uses a default converter that reserves the maximum range required for
    /// each character, so for instance 256 for a byte (u8) character.
    pub fn new() -> Self {
        Self {
            converter: IdConverter::new::<T>(),
            sampling_level: None,
            _t: std::marker::PhantomData,
        }
    }
}

impl<T> Default for SearchIndexBuilder<T, IdConverter>
where
    T: Character,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, C> SearchIndexBuilder<T, C>
where
    T: Character,
    C: Converter<T>,
{
    /// Construct a new search index builder with a custom converter.
    ///
    /// This way you can restrict the range of characters used in the text,
    /// using less memory.
    ///
    /// Use [`converter::RangeConverter`] if you can constrain characters to a
    /// particular range.
    pub fn with_converter(converter: C) -> Self {
        Self {
            converter,
            sampling_level: None,
            _t: std::marker::PhantomData,
        }
    }

    /// Set the sampling level for the suffix array stored in the index.
    ///
    /// A sampling level of 0 means the most memory is used (a full suffix-array is
    /// stored), while a higher sampling level means less memory is used but the
    /// `locate` operation is slower. Each increase of a sampling level uses half the
    /// memory.
    ///
    /// Note that you cannot combine this with `count_only`.
    pub fn sampling_level(self, level: usize) -> Self {
        Self {
            sampling_level: Some(level),
            ..self
        }
    }

    /// Use a special search index that uses run length encoding.
    ///
    /// This is more memory efficient than the default [`FMIndex] index, but
    /// slower.
    pub fn run_length_encoding(self) -> RLFMSearchIndexBuilder<T, C> {
        RLFMSearchIndexBuilder {
            converter: self.converter,
            sampling_level: self.sampling_level,
            _t: std::marker::PhantomData,
        }
    }

    /// The index only supports the count information.
    ///
    /// This means you cannot use `locate` on search, but it uses less memory.
    pub fn count_only(self) -> CountOnlySearchIndexBuilder<T, C> {
        // unfortunately I can't think of a nice type-driven way to do this,
        // as we cannot make default count-only and fit our guidelines
        if self.sampling_level.is_some() {
            panic!("Cannot use sampling level with count-only index");
        }
        CountOnlySearchIndexBuilder {
            converter: self.converter,
            _t: std::marker::PhantomData,
        }
    }

    /// Build the index.
    pub fn build(self, text: Vec<T>) -> FMIndexLocateSearchIndex<T, C> {
        FMIndex::new(text, self.converter, self.sampling_level.unwrap_or(0))
    }
}

/// Builder for search indexes that only support the count operation, not locate.
pub struct CountOnlySearchIndexBuilder<T, C>
where
    T: Character,
    C: Converter<T>,
{
    converter: C,
    _t: std::marker::PhantomData<T>,
}

impl<T, C> CountOnlySearchIndexBuilder<T, C>
where
    T: Character,
    C: Converter<T>,
{
    /// Use the run-length encoding index.
    pub fn run_length_encoding(self) -> RLFMCountOnlySearchIndexBuilder<T, C> {
        RLFMCountOnlySearchIndexBuilder {
            converter: self.converter,
            _t: std::marker::PhantomData,
        }
    }

    /// Build the index.
    pub fn build(self, text: Vec<T>) -> FMIndexCountOnlySearchIndex<T, C> {
        FMIndex::count_only(text, self.converter)
    }
}

/// Builder for an index that uses run-length encoding.
pub struct RLFMSearchIndexBuilder<T, C>
where
    T: Character,
    C: Converter<T>,
{
    converter: C,
    sampling_level: Option<usize>,
    _t: std::marker::PhantomData<T>,
}

impl<T, C> RLFMSearchIndexBuilder<T, C>
where
    T: Character,
    C: Converter<T>,
{
    /// Set the sampling level for the suffix array stored in the index.
    ///
    /// A sampling level of 0 means the most memory is used (a full suffix-array is
    /// stored), while a higher sampling level means less memory is used but the
    /// `locate` operation is slower. Each increase of a sampling level uses half the
    /// memory.
    ///
    /// Note that you cannot combine this with `count_only`
    pub fn sampling_level(self, level: usize) -> Self {
        Self {
            sampling_level: Some(level),
            ..self
        }
    }

    /// The index only supports the count information.
    ///
    /// This means you cannot use `locate` on search, but it uses less memory.
    pub fn count_only(self) -> RLFMCountOnlySearchIndexBuilder<T, C> {
        // unfortunately I can't think of a nice type-driven way to do this,
        // as we cannot make default count-only and fit our guidelines
        if self.sampling_level.is_some() {
            panic!("Cannot use sampling level with count-only index");
        }
        RLFMCountOnlySearchIndexBuilder {
            converter: self.converter,
            _t: std::marker::PhantomData,
        }
    }

    /// Build the index.
    pub fn build(self, text: Vec<T>) -> RLFMIndexLocateSearchIndex<T, C> {
        RLFMIndex::new(text, self.converter, self.sampling_level.unwrap_or(0))
    }
}

/// Builder for an index that uses run-length encoding and only supports count queries.
pub struct RLFMCountOnlySearchIndexBuilder<T, C>
where
    T: Character,
    C: Converter<T>,
{
    converter: C,
    _t: std::marker::PhantomData<T>,
}

impl<T, C> RLFMCountOnlySearchIndexBuilder<T, C>
where
    T: Character,
    C: Converter<T>,
{
    /// Build the index.
    pub fn build(self, text: Vec<T>) -> RLFMIndexCountOnlySearchIndex<T, C> {
        RLFMIndex::count_only(text, self.converter)
    }
}
