//! This crate provides implementations of *FM-Index* and its variants.
//!
//! FM-Index, originally proposed by Paolo Ferragina and Giovanni Manzini [^1],
//! is a compressed full-text index which supports the following queries:
//!
//! - `count`: Given a pattern string, counts the number of its occurrences.
//! - `locate`: Given a pattern string, lists all positions of its occurrences.
//! - `extract`: Given an integer, gets the character of the text at that
//!   position.
//!
//! `fm-index` crate does not support the third query (extracting a character
//! from arbitrary position). Instead, it provides backward/forward iterators
//! that return the text characters starting from a search result.
//!
//! # Usage
//!
//! Add this to your `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! fm-index = "0.2.0"
//! ```
//!
//! # Example
//! ```
//! use fm_index::FMIndexWithLocate;
//! use fm_index::converter::RangeConverter;
//!
//! // Prepare a text string to search for patterns.
//! let text = concat!(
//!     "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
//!     "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
//!     "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.",
//!     "Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
//! ).as_bytes().to_vec();
//!
//! // Converter converts each character into packed representation.
//! // `' '` ~ `'~'` represents a range of ASCII printable characters.
//! let converter = RangeConverter::new(b' ', b'~');
//!
//! // To perform locate queries, we need to use some storage.
//! // the sampling level determines how much is retained in order to support `locate`
//! // queries. `0` retains the full information, but we don't need the whole array
//! // since we can interpolate missing elements in a suffix array from others. A sampler
//! // will _sieve_ a suffix array for this purpose. If you don't need `locate` queries
//! // you can save the memory by not setting a sampling level.
//! let index = FMIndexWithLocate::new(text, converter, 2);
//!
//! // Search for a pattern string.
//! let pattern = "dolor";
//! let search = index.search(pattern);
//!
//! // Count the number of occurrences.
//! let n = search.count();
//! assert_eq!(n, 4);
//!
//! // List the position of all occurrences.
//! let positions = search.locate();
//! assert_eq!(positions, vec![246, 12, 300, 103]);
//!
//! // Extract preceding characters from a search position.
//! let i = 0;
//! let mut prefix = search.iter_backward(i).take(16).collect::<Vec<u8>>();
//! prefix.reverse();
//! assert_eq!(prefix, b"Duis aute irure ".to_owned());
//!
//! // Extract succeeding characters from a search position.
//! let i = 3;
//! let postfix = search.iter_forward(i).take(20).collect::<Vec<u8>>();
//! assert_eq!(postfix, b"dolore magna aliqua.".to_owned());
//!
//! // Search can be chained backward.
//! let search_chained = search.search("et ");
//! assert_eq!(search_chained.count(), 1);
//! ```
//!
//! # Implementations
//!
//! ## FM-Index
//!
//! This is a fast implementation of a succinct full-text index. For most use
//! cases this is the best choice.
//!
//! The implementation is based on [^1]. The index is constructed with a suffix
//! array generated by SA-IS [^3] in _O(n)_ time, where _n_ is the size of a
//! text string.
//!
//! Basically it consists of
//!
//! - a Burrows-Wheeler transform (BWT) of the text string represented with
//!   _wavelet matrix_ [^4]
//! - an array of size _O(σ)_ (_σ_: number of characters) which stores the
//!   number of characters smaller than a given character
//! - a (sampled) suffix array
//!
//! ## Run-Length FM-Index
//!
//! This index is more compact than FM-Index if there are a lot of repeated
//! patterns in the text. It is slower than FM-Index in general.
//!
//! Based on [^2]. The index is constructed with a suffix array generated by
//! SA-IS [^3].
//!
//! It consists of
//!
//! - a wavelet matrix that stores the run heads of BWT of the text string
//! - a succinct bit vector which stores the run lengths of BWT of the text
//!   string
//! - a succinct bit vector which stores the run lengths of BWT of the text
//!   string sorted in alphabetical order of corresponding run heads
//! - an array of size _O(σ)_ (_σ_: number of characters) which stores the
//!   number of characters smaller than a given character in run heads
//!
//! # Reference
//!
//! [^1]: Ferragina, P., & Manzini, G. (2000). Opportunistic data structures
//!     with applications. Annual Symposium on Foundations of Computer Science
//!     \- Proceedings, 390–398. <https://doi.org/10.1109/sfcs.2000.892127>
//!
//! [^2]: Mäkinen, V., & Navarro, G. (2005). Succinct suffix arrays based on
//!     run-length encoding. In Lecture Notes in Computer Science (Vol. 3537).
//!     <https://doi.org/10.1007/11496656_5>
//!
//! [^3]: Ge Nong, Sen Zhang, & Wai Hong Chan. (2010). Two Efficient Algorithms
//!     for Linear Time Suffix Array Construction. IEEE Transactions on
//!     Computers, 60(10), 1471–1484. <https://doi.org/10.1109/tc.2010.188>
//!
//! [^4]: Claude F., Navarro G. (2012). The Wavelet Matrix. In:
//!     Calderón-Benavides L., González-Caro C., Chávez E., Ziviani N. (eds)
//!     String Processing and Information Retrieval. SPIRE 2012.
//!     <https://doi.org/10.1007/978-3-642-34109-0_18>
#![allow(clippy::len_without_is_empty)]
#![warn(missing_docs)]

pub mod converter;

mod backend;
mod character;
mod fm_index;
mod frontend;
mod multi_text;
mod rlfmi;
mod suffix_array;
mod util;
mod wrapper;

pub use backend::HeapSize;
pub use character::Character;
pub use frontend::{
    FMIndex, FMIndexSearch, FMIndexSearchWithLocate, FMIndexWithLocate, MultiTextFMIndex,
    MultiTextFMIndexSearch, MultiTextFMIndexSearchWithLocate, MultiTextFMIndexWithLocate,
    RLFMIndex, RLFMIndexSearch, RLFMIndexSearchWithLocate, RLFMIndexWithLocate, Search,
    SearchIndex, SearchIndexWithLocate, SearchWithLocate,
};
