use num_traits::Zero;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

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

pub struct TestRunner {
    pub texts: usize,
    pub patterns: usize,
    pub text_size: usize,
    pub alphabet_size: u8,
    pub level_max: usize,
    pub pattern_size_max: usize,
}

impl TestRunner {
    pub fn run<I, B, R>(&self, build_index: B, run_test: R)
    where
        B: Fn(Vec<u8>, usize) -> I,
        R: Fn(&I, &[u8], &[u8]),
    {
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..self.texts {
            let text = build_text(
                || rng.gen::<u8>() % (self.alphabet_size as u8),
                self.text_size,
            );
            let level = rng.gen::<usize>() % (self.level_max + 1);
            let fm_index = build_index(text.clone(), level);

            for _ in 0..self.patterns {
                let pattern_size = rng.gen::<usize>() % (self.pattern_size_max - 1) + 1;
                let pattern = (0..pattern_size)
                    .map(|_| rng.gen::<u8>() % (self.alphabet_size - 1) + 1)
                    .collect::<Vec<_>>();

                run_test(&fm_index, &text, &pattern);
            }
        }
    }
}
