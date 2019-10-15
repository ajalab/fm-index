use fm_index::converter::{Converter, RangeConverter};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub fn binary_text_set(len: usize, prob: f64) -> (Vec<u8>, impl Converter<u8>) {
    let zero = b'0';
    let one = b'1';
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let mut text = (0..len)
        .map(|_| if rng.gen_bool(prob) { zero } else { one })
        .collect::<Vec<_>>();
    text.push(0);

    let converter = RangeConverter::new(zero, one);
    (text, converter)
}

pub fn binary_patterns(m: usize) -> Vec<String> {
    let mut patterns: Vec<String> = vec!["".to_owned()];
    for _ in 0..m {
        patterns = patterns
            .into_iter()
            .flat_map(|s| vec![s.clone() + "0", s + "1"])
            .collect();
    }
    patterns
}