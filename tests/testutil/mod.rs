use fm_index::{Error, PieceId, Text};
use num_traits::Zero;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Build a text for tests using a generator function `gen`.
pub fn build_text<T: Zero + Clone, F: FnMut() -> T>(mut gen: F, len: usize) -> Vec<T> {
    debug_assert!(len > 0, "Text length must be greater than 0");

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

pub struct NaiveSearchIndex<'a> {
    text: &'a [u8],
}

impl<'a> NaiveSearchIndex<'a> {
    pub fn new(text: &'a [u8]) -> Self {
        NaiveSearchIndex { text }
    }

    pub fn search(&self, pattern: &[u8]) -> Vec<NaiveSearchIndexMatch> {
        self.do_search(pattern, false, false)
    }

    #[allow(dead_code)] // false positive?
    pub fn search_prefix(&self, pattern: &[u8]) -> Vec<NaiveSearchIndexMatch> {
        self.do_search(pattern, true, false)
    }

    #[allow(dead_code)] // false positive?
    pub fn search_suffix(&self, pattern: &[u8]) -> Vec<NaiveSearchIndexMatch> {
        self.do_search(pattern, false, true)
    }

    #[allow(dead_code)] // false positive?
    pub fn search_exact(&self, pattern: &[u8]) -> Vec<NaiveSearchIndexMatch> {
        self.do_search(pattern, true, true)
    }

    fn do_search(
        &self,
        pattern: &[u8],
        match_prefix_only: bool,
        match_suffix_only: bool,
    ) -> Vec<NaiveSearchIndexMatch> {
        let mut result = Vec::new();
        let mut piece_id = 0;
        for i in 0..self.text.len() - pattern.len() + 1 {
            if self.text[i] == 0 {
                piece_id += 1;
            }
            if (!match_prefix_only || (i == 0 || self.text[i - 1] == 0))
                && (!match_suffix_only
                    || (i + pattern.len() == self.text.len() || self.text[i + pattern.len()] == 0))
                && &self.text[i..i + pattern.len()] == pattern
            {
                result.push(NaiveSearchIndexMatch {
                    position: i,
                    piece_id: PieceId::from(piece_id),
                });
            }
        }
        result
    }
}

pub struct NaiveSearchIndexMatch {
    pub position: usize,
    #[allow(dead_code)] // false positive?
    pub piece_id: PieceId,
}

pub struct TestRunner {
    pub texts: usize,
    pub patterns: usize,
    pub text_size_max: usize,
    pub alphabet_size: u8,
    pub level_max: usize,
    pub pattern_size_max: usize,
    pub multi_pieces: bool,
}

impl TestRunner {
    pub fn run<I, B, R>(&self, build_index: B, run_test: R)
    where
        B: Fn(&Text<u8, Vec<u8>>, usize) -> Result<I, Error>,
        R: Fn(&I, &Text<u8, Vec<u8>>, &[u8]),
    {
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..self.texts {
            let text_size = rng.gen::<usize>() % (self.text_size_max) + 1;
            let text = if self.multi_pieces {
                build_text(|| rng.gen::<u8>() % self.alphabet_size, text_size)
            } else {
                build_text(|| rng.gen::<u8>() % self.alphabet_size + 1, text_size)
            };
            let text = Text::new(text);
            let level = rng.gen::<usize>() % (self.level_max + 1);
            let fm_index = build_index(&text, level).unwrap();

            for _ in 0..self.patterns {
                let pattern_size_max = if self.pattern_size_max > text_size {
                    text_size
                } else {
                    self.pattern_size_max
                };
                let pattern_size = rng.gen::<usize>() % (pattern_size_max - 1) + 1;
                let pattern = (0..pattern_size)
                    .map(|_| rng.gen::<u8>() % (self.alphabet_size - 1) + 1)
                    .collect::<Vec<_>>();

                run_test(&fm_index, &text, &pattern);
            }
        }
    }
}
