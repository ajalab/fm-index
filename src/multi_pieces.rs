use std::ops::{Rem, Sub};

use crate::backend::{HasMultiPieces, HasPosition, SearchIndexBackend};
use crate::character::Character;
use crate::piece::PieceId;
use crate::suffix_array::sais;
use crate::suffix_array::sample::SuffixOrderSampledArray;
use crate::text::Text;
use crate::HeapSize;

use serde::{Deserialize, Serialize};
use vers_vecs::{BitVec, RsVec, WaveletMatrix};

// An FM-Index supporting multiple \0 separated texts
#[derive(Serialize, Deserialize)]
pub struct FMIndexMultiPiecesBackend<C, S> {
    bw: WaveletMatrix,
    cs: Vec<usize>,
    suffix_array: S,
    doc: Vec<usize>,
    // The index of the first text in the suffix array
    sa_idx_first_text: usize,
    _c: std::marker::PhantomData<C>,
}

impl<C, S> FMIndexMultiPiecesBackend<C, S>
where
    C: Character,
{
    pub(crate) fn new<T>(text: &Text<C, T>, get_sample: impl Fn(&[usize]) -> S) -> Self
    where
        T: AsRef<[C]>,
    {
        let cs = sais::get_bucket_start_pos(&sais::count_chars(text));
        let sa = sais::build_suffix_array(text);
        let bw = Self::wavelet_matrix(text, &sa);
        let (doc, sa_idx_first_text) = Self::doc(text.text(), &bw, &sa);

        FMIndexMultiPiecesBackend {
            cs,
            bw,
            suffix_array: get_sample(&sa),
            doc,
            sa_idx_first_text,
            _c: std::marker::PhantomData::<C>,
        }
    }

    fn doc(text: &[C], bw: &WaveletMatrix, sa: &[usize]) -> (Vec<usize>, usize) {
        let mut end_marker_bits = BitVec::from_zeros(text.len());
        let mut end_marker_count = 0;
        for (i, c) in text.iter().enumerate() {
            if c.into_u64() == 0 {
                end_marker_bits.set(i, 1).unwrap();
                end_marker_count += 1;
            }
        }
        let end_marker_flags = RsVec::from_bit_vec(end_marker_bits);

        let mut end_marker_rank_l = 0;
        let mut doc = vec![0; end_marker_count];
        let mut sa_idx_first_text = 0;
        while let Some(p) = bw.select_u64(end_marker_rank_l, 0) {
            let end_marker_idx = modular_sub(sa[p], 1, sa.len());
            let piece_id = end_marker_flags.rank1(end_marker_idx);
            if piece_id == end_marker_count - 1 {
                sa_idx_first_text = p;
            }
            doc[end_marker_rank_l] = piece_id;

            end_marker_rank_l += 1;
        }

        (doc, sa_idx_first_text)
    }

    fn wavelet_matrix<T>(text: &Text<C, T>, sa: &[usize]) -> WaveletMatrix
    where
        T: AsRef<[C]>,
    {
        let n = text.text().len();
        let mut bw = vec![0u64; n];
        for i in 0..n {
            let k = sa[i];
            if k > 0 {
                bw[i] = text.text()[k - 1].into_u64();
            }
        }
        let bw = bw.into_iter().collect::<Vec<u64>>();

        WaveletMatrix::from_slice(&bw, text.max_bits() as u16)
    }
}

impl<C> HeapSize for FMIndexMultiPiecesBackend<C, ()>
where
    C: Character,
{
    fn heap_size(&self) -> usize {
        self.bw.heap_size() + self.cs.capacity() * std::mem::size_of::<u64>()
    }
}

impl<C> HeapSize for FMIndexMultiPiecesBackend<C, SuffixOrderSampledArray>
where
    C: Character,
{
    fn heap_size(&self) -> usize {
        self.bw.heap_size()
            + self.cs.capacity() * std::mem::size_of::<u64>()
            + self.suffix_array.size()
            + self.doc.capacity() * std::mem::size_of::<usize>()
    }
}

impl<C, S> SearchIndexBackend for FMIndexMultiPiecesBackend<C, S>
where
    C: Character,
{
    type C = C;

    fn len(&self) -> usize {
        self.bw.len()
    }

    fn get_l(&self, i: usize) -> Self::C {
        Self::C::from_u64(self.bw.get_u64_unchecked(i))
    }

    fn lf_map(&self, i: usize) -> usize {
        let c = self.get_l(i);
        let rank = self.bw.rank_u64_unchecked(i, c.into_u64());
        if c.into_u64() == 0 {
            match i.cmp(&self.sa_idx_first_text) {
                std::cmp::Ordering::Less => rank + 1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => rank,
            }
        } else {
            let c_count = self.cs[c.into_usize()];
            rank + c_count
        }
    }

    fn lf_map2(&self, c: C, i: usize) -> usize {
        let rank = self.bw.rank_u64_unchecked(i, c.into_u64());
        if c.into_u64() == 0 {
            match i.cmp(&self.sa_idx_first_text) {
                std::cmp::Ordering::Less => rank + 1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => rank,
            }
        } else {
            let c_count = self.cs[c.into_usize()];
            rank + c_count
        }
    }

    fn get_f(&self, i: usize) -> Self::C {
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
        C::from_usize(s)
    }

    fn fl_map(&self, i: usize) -> Option<usize> {
        let c = self.get_f(i);
        if c.into_u64() == 0 {
            None
        } else {
            Some(
                self.bw
                    .select_u64_unchecked(i - self.cs[c.into_usize()], c.into_u64()),
            )
        }
    }
}

impl<C> HasPosition for FMIndexMultiPiecesBackend<C, SuffixOrderSampledArray>
where
    C: Character,
{
    fn get_sa(&self, mut i: usize) -> usize {
        let mut steps = 0;
        loop {
            match self.suffix_array.get(i) {
                Some(sa) => {
                    return (sa + steps) % self.bw.len();
                }
                None => {
                    i = self.lf_map(i);
                    steps += 1;
                }
            }
        }
    }
}

impl<C, S> HasMultiPieces for FMIndexMultiPiecesBackend<C, S>
where
    C: Character,
{
    fn piece_id(&self, mut i: usize) -> PieceId {
        loop {
            if self.get_l(i).into_u64() == 0 {
                let piece_id_prev = self.doc[self.bw.rank_u64_unchecked(i, 0)];
                let piece_id = modular_add(piece_id_prev, 1, self.doc.len());
                return PieceId::from(piece_id);
            } else {
                i = self.lf_map(i);
            }
        }
    }

    fn pieces_count(&self) -> usize {
        self.doc.len()
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
    use crate::suffix_array::sample::SuffixOrderSampledArray;
    use crate::testutil;
    use rand::{rngs::StdRng, Rng, SeedableRng};

    #[test]
    fn test_lf_map_random() {
        let text_size = 512;
        let attempts = 100;
        let alphabet_size = 8;
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..attempts {
            let text = testutil::build_text(|| rng.gen::<u8>() % alphabet_size, text_size);
            let suffix_array = testutil::build_suffix_array(&text);
            let inv_suffix_array = testutil::build_inv_suffix_array(&suffix_array);
            let fm_index = FMIndexMultiPiecesBackend::new(&Text::new(text), |sa| {
                SuffixOrderSampledArray::sample(sa, 0)
            });

            let mut lf_map_expected = vec![0; text_size];
            let mut lf_map_actual = vec![0; text_size];
            for i in 0..text_size {
                let k = modular_sub(suffix_array[i], 1, text_size);
                lf_map_expected[i] = inv_suffix_array[k];
                lf_map_actual[i] = fm_index.lf_map(i);
            }

            assert_eq!(lf_map_expected, lf_map_actual);
        }
    }

    #[test]
    fn test_get_piece_id() {
        let text = "foo\0bar\0baz\0".as_bytes();
        let suffix_array = testutil::build_suffix_array(text);
        let fm_index = FMIndexMultiPiecesBackend::new(&Text::new(text), |sa| {
            SuffixOrderSampledArray::sample(sa, 0)
        });

        for (i, &char_pos) in suffix_array.iter().enumerate() {
            let piece_id_expected =
                PieceId::from(text[..char_pos].iter().filter(|&&c| c == 0).count());
            let piece_id_actual = fm_index.piece_id(i);
            assert_eq!(
                piece_id_expected, piece_id_actual,
                "the piece ID of a character at position {} ({} in suffix array) must be {:?}",
                char_pos, i, piece_id_expected
            );
        }
    }

    #[test]
    fn test_get_piece_id_random() {
        let text_size = 512;
        let attempts = 100;
        let alphabet_size = 8;
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..attempts {
            let text = testutil::build_text(|| rng.gen::<u8>() % alphabet_size, text_size);
            let suffix_array = testutil::build_suffix_array(&text);
            let fm_index = FMIndexMultiPiecesBackend::new(&Text::new(&text), |sa| {
                SuffixOrderSampledArray::sample(sa, 0)
            });

            for (i, &char_pos) in suffix_array.iter().enumerate() {
                let piece_id_expected =
                    PieceId::from(text[..(char_pos)].iter().filter(|&&c| c == 0).count());
                let piece_id_actual = fm_index.piece_id(i);
                assert_eq!(
                    piece_id_expected, piece_id_actual,
                    "the piece ID of a character at position {} ({} in suffix array) must be {:?}. text={:?}, suffix_array={:?}",
                    char_pos, i, piece_id_expected, text, suffix_array,
                );
            }
        }
    }
}
