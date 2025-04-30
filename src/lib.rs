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
//! # Implementations
//!
//! This section describes the implementations of FM-Index and its variants.
//!
//! Throughout this documentation, we use the following notations:
//!
//! - _n_: the size of a text string
//! - _σ_: the number of bits needed to represent the characters in the text.
//! This can be controlled by [`Text::max_character`].
//!
//! ## FM-Index ([`FMIndex`], [`FMIndexWithLocate`])
//!
//! This is a standard FM-index suitable for single-text use cases. It offers
//! a good balance between space efficiency and query performance, making it
//! a strong default choice for most applications.
//!
//! The implementation is based on Ferragina and Manzini's original work [^1].
//! The suffix array is constructed using the SA-IS algorithm [^3] in _O(n)_ time.
//!
//! The data structure consists of the following components:
//!
//! - A wavelet matrix ([`vers_vecs::WaveletMatrix`]) that stores the Burrows–Wheeler
//!   Transform (BWT) of the text. Its space complexity is _O(n log σ)_ bits.
//! - A `Vec<usize>` of length _O(σ)_, which stores the number of characters
//!   smaller than a given character in the text.
//! - (Only in [`FMIndexWithLocate`]) A sampled suffix array of length _O(n / 2^l)_,
//!   used to determine the positions of pattern occurrences.
//!
//! ## Run-Length FM-Index ([`RLFMIndex`], [`RLFMIndexWithLocate`])
//!
//! This variant is optimized for highly repetitive texts. It offers better compression
//! than the standard FM-index when the Burrows–Wheeler Transform contains long runs
//! of repeated characters. However, it is generally slower in query performance.
//!
//! The implementation is based on run-length encoded FM-indexing [^2]. The suffix
//! array is constructed with the SA-IS algorithm [^3] in _O(n)_ time.
//!
//! The data structure consists of the following components:
//!
//! - A wavelet matrix ([`vers_vecs::WaveletMatrix`]) that stores the run heads
//!   of the BWT. The space complexity is _O(r log σ)_ bits, where _r_ is the number
//!   of runs in the BWT.
//! - A succinct bit vector that encodes the run lengths of the BWT.
//! - A second succinct bit vector that encodes the run lengths of the BWT
//!   when sorted by the lexicographic order of run heads.
//! - A `Vec<usize>` of length _O(σ)_, which stores the number of characters
//!   smaller than a given character in the run heads.
//! - (Only in [`RLFMIndexWithLocate`]) A sampled suffix array of length _O(n / 2^l)_,
//!   used to determine the positions of pattern occurrences.
//!
//! ## FM-Index for Multiple Texts ([`FMIndexMultiPieces`], [`FMIndexMultiPiecesWithLocate`])
//!
//! This index is designed for multiple texts (text pieces) separated by a null character (`\0`).
//! The implementation is based on SXSI [^5].
//!
//! Each text piece is assigned a unique ID ([`PieceId`]).
//! The index supports locating the text piece ID for each search result.
//! It also supports searching for patterns that are prefixes or suffixes of
//! individual text pieces.
//!
//! The data structure consists of the following components:
//!
//! - A wavelet matrix ([`vers_vecs::WaveletMatrix`]) that stores the concatenated
//!   text pieces, separated by null characters (`\0`). The space complexity is
//!   _O(n log σ)_ bits.
//! - A `Vec<usize>` of length _O(σ)_, which stores the number of characters
//!   smaller than a given character.
//! - A `Vec<usize>` of length _O(d)_, where _d_ is the number of text
//!   pieces. This array maps between the suffix array and the text piece IDs.
//! - (Only in [`FMIndexMultiPiecesWithLocate`]) A sampled suffix array for the text pieces.
//!   Its length is _O(n / 2^l)_, and it is used to determine the position
//!   of each pattern occurrence in the text.
//!
//! [^1]: Ferragina, P., & Manzini, G. (2000). Opportunistic data structures
//!     with applications. Proceedings 41st Annual Symposium on Foundations
//!     of Computer Science, 390–398. <https://doi.org/10.1109/SFCS.2000.892127>
//!
//! [^2]: Mäkinen, V., & Navarro, G. (2005). Succinct suffix arrays based on
//!     run-length encoding. In Combinatorial Pattern Matching: 16th Annual Symposium,
//!     CPM 2005, Jeju Island, Korea, June 19-22, 2005. Proceedings 16 (pp. 45-56).
//!     Springer Berlin Heidelberg. <https://doi.org/10.1007/11496656_5>
//!
//! [^3]: Nong, G., Zhang, S., & Chan, W. H. (2010). Two efficient algorithms
//!     for linear time suffix array construction. IEEE transactions on computers,
//!     60(10), 1471-1484. <https://doi.org/10.1109/tc.2010.188>
//!
//! [^4]: Claude F., Navarro G. (2012). The Wavelet Matrix. In:
//!     Calderón-Benavides L., González-Caro C., Chávez E., Ziviani N. (eds)
//!     String Processing and Information Retrieval. SPIRE 2012.
//!
//! [^4]: Claude, F., & Navarro, G. (2012). The wavelet matrix.
//!     In International Symposium on String Processing and Information Retrieval (pp. 167-179).
//!     Berlin, Heidelberg: Springer Berlin Heidelberg.
//!     <https://doi.org/10.1007/978-3-642-34109-0_18>
//!
//! [^5]: Arroyuelo, A., Claude, F., Maneth, S., Mäkinen, V., Navarro, G., Nguyen,
//!     K., Siren, J., & Välimäki, N. (2011). Fast In-Memory XPath Search over
//!     Compressed Text and Tree Indexes (No. arXiv:0907.2089).
//!     arXiv. <https://doi.org/10.48550/arXiv.0907.2089>

#![allow(clippy::len_without_is_empty)]
#![warn(missing_docs)]

mod backend;
mod character;
mod error;
mod fm_index;
mod frontend;
mod heap_size;
mod multi_pieces;
mod piece;
mod rlfmi;
mod suffix_array;
#[cfg(test)]
mod testutil;
mod text;
mod util;
mod wrapper;

pub use character::Character;
pub use error::Error;
pub use frontend::{
    FMIndex, FMIndexMatch, FMIndexMatchWithLocate, FMIndexMultiPieces, FMIndexMultiPiecesMatch,
    FMIndexMultiPiecesMatchWithLocate, FMIndexMultiPiecesSearch,
    FMIndexMultiPiecesSearchWithLocate, FMIndexMultiPiecesWithLocate, FMIndexSearch,
    FMIndexSearchWithLocate, FMIndexWithLocate, Match, MatchWithLocate, MatchWithPieceId,
    RLFMIndex, RLFMIndexMatch, RLFMIndexMatchWithLocate, RLFMIndexSearch,
    RLFMIndexSearchWithLocate, RLFMIndexWithLocate, Search, SearchIndex,
    SearchIndexWithMultiPieces,
};
pub use piece::PieceId;
pub use text::Text;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
