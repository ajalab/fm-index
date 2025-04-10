use crate::character::Character;
use num_traits::Zero;

/// Build a text for tests using a generator function `gen`.
pub fn build_text<C: Zero + Clone, F: FnMut() -> C>(mut gen: F, len: usize) -> Vec<C> {
    let mut text = vec![C::zero(); len];

    let mut prev_zero = true;
    for t in text.iter_mut().take(len - 1) {
        let mut c = gen();
        if prev_zero {
            while c.is_zero() {
                c = gen();
            }
        }
        prev_zero = c.is_zero();
        *t = c;
    }

    while text[len - 2].is_zero() {
        text[len - 2] = gen();
    }

    text
}

/// Compute the suffix array of the given text in naive way for testing purpose.
pub fn build_suffix_array<C>(text: &[C]) -> Vec<usize>
where
    C: Character + Ord,
{
    let n = text.len();
    let suffixes = (0..n).map(|i| &text[i..n]).collect::<Vec<_>>();
    let mut sa = (0..(suffixes.len())).collect::<Vec<_>>();
    sa.sort_by_key(|i| &suffixes[*i]);
    sa
}

/// Build the inverse suffix array from the suffix array.
pub fn build_inv_suffix_array(suffix_array: &[usize]) -> Vec<usize> {
    let mut isa = vec![0; suffix_array.len()];
    for (p, &i) in suffix_array.iter().enumerate() {
        isa[i] = p;
    }
    isa
}
