use num_traits::Zero;

/// Build a text for tests using a generator function `gen`.
pub(crate) fn build_text<T: Zero, F: FnMut() -> T>(mut gen: F, len: usize) -> Vec<T> {
    let mut text = (0..(len - 1)).map(|_| gen()).collect::<Vec<_>>();

    // Truncate the trailing zeros, since SA-IS does not support trailing zero suffix longer than 1.
    if let Some(pos) = text.iter().rposition(|x| !x.is_zero()) {
        text.truncate(pos + 1);
    } else {
        text.clear();
    }

    // Add non-zero elements until the text length reaches len - 1.
    while text.len() < len - 1 {
        let c = gen();
        if !c.is_zero() {
            text.push(c);
        }
    }

    // Add the last zero as a sentinel for SA-IS.
    text.push(T::zero());
    text
}
