//! This crate provides implementations of FM Index and its variants.
//!
//! *FM Index*, originally proposed by Paolo Ferragina and Giovanni Manzini,
//! is a compressed full-text index which supports the following queries:
//!
//! - `count`: Given a pattern string, counts the number of its occurrences.
//! - `locate`: Given a pattern string, lists the all position of its occurrences.
//! - `extract`: Given an integer, gets the character of the text at that position.
//!
//! `fm-index` crate does not support the third query (extracting a character from arbitrary position) now.
//! Instead, it provides an extraction query from a search result.

#![allow(clippy::len_without_is_empty)]

mod character;
pub mod converter;
mod fm_index;
mod iter;
mod rlfmi;
mod sais;
mod search;
pub mod suffix_array;
mod util;
mod wavelet_matrix;

pub use crate::fm_index::FMIndex;
pub use crate::rlfmi::RLFMIndex;

pub use iter::{BackwardIterableIndex, ForwardIterableIndex};
pub use search::BackwardSearchIndex;
pub use suffix_array::IndexWithSA;
