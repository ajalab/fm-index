use num_traits::Zero;

/// Build a text for tests using a generator function `gen`.
pub fn build_text<T: Zero + Clone, F: FnMut() -> T>(mut gen: F, len: usize) -> Vec<T> {
    let mut text = vec![T::zero(); len];

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
