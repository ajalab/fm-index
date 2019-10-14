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
