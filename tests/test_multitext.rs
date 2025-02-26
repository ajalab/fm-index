use rand::{rngs::StdRng, Rng, SeedableRng};

mod testutil;
use fm_index::converter::IdConverter;
use fm_index::{MatchWithLocate, MatchWithTextId, MultiTextFMIndexWithLocate, Search, TextId};

#[test]
fn test_search_count() {
    let text_size = 1024;
    let alphabet_size = 8;
    let pattern_size_max = 128;
    let text = generate_text_random(text_size, alphabet_size);

    let fm_index = MultiTextFMIndexWithLocate::new(text.clone(), IdConverter::new::<u8>(), 0);

    let mut rng = StdRng::seed_from_u64(0);
    for i in 0..1000 {
        let pattern_size = rng.gen::<usize>() % (pattern_size_max - 1) + 1;
        let pattern = (0..pattern_size)
            .map(|_| rng.gen::<u8>() % (alphabet_size - 1) + 1)
            .collect::<Vec<_>>();

        let mut count_expected = 0;
        for i in 0..=(text_size - pattern_size) {
            if text[i..i + pattern_size] == pattern {
                count_expected += 1;
            }
        }
        let count_actual = fm_index.search(&pattern).count();

        assert_eq!(
            count_expected, count_actual,
            "i = {:?}, text = {:?}, pattern = {:?}",
            i, text, pattern
        );
    }
}

#[test]
fn test_search_locate() {
    let text_size = 1024;
    let alphabet_size = 8;
    let pattern_size_max = 128;
    let text = generate_text_random(text_size, alphabet_size);

    let fm_index = MultiTextFMIndexWithLocate::new(text.clone(), IdConverter::new::<u8>(), 0);

    let mut rng = StdRng::seed_from_u64(0);
    for i in 0..1000 {
        let pattern_size = rng.gen::<usize>() % (pattern_size_max - 1) + 1;
        let pattern = (0..pattern_size)
            .map(|_| rng.gen::<u8>() % (alphabet_size - 1) + 1)
            .collect::<Vec<_>>();

        let mut positions_expected = Vec::new();
        for i in 0..=(text_size - pattern_size) {
            if text[i..i + pattern_size] == pattern {
                positions_expected.push(i as u64);
            }
        }
        let mut positions_actual = fm_index.search(&pattern).locate();
        positions_actual.sort();

        assert_eq!(
            positions_expected, positions_actual,
            "i = {:?}, text = {:?}, pattern = {:?}",
            i, text, pattern
        );
    }
}

#[test]
fn test_search_iter_matches_locate() {
    let text_size = 1024;
    let alphabet_size = 8;
    let pattern_size_max = 128;

    let mut rng = StdRng::seed_from_u64(0);
    let text = testutil::build_text(|| rng.gen::<u8>() % alphabet_size, text_size);

    let fm_index = MultiTextFMIndexWithLocate::new(text.clone(), IdConverter::new::<u8>(), 0);

    for i in 0..1000 {
        let pattern_size = rng.gen::<usize>() % (pattern_size_max - 1) + 1;
        let pattern = (0..pattern_size)
            .map(|_| rng.gen::<u8>() % (alphabet_size - 1) + 1)
            .collect::<Vec<_>>();

        let mut positions_expected = Vec::new();
        for i in 0..=(text_size - pattern_size) {
            if text[i..i + pattern_size] == pattern {
                positions_expected.push(i as u64);
            }
        }
        let mut positions_actual = fm_index
            .search(&pattern)
            .iter_matches()
            .map(|m| m.locate())
            .collect::<Vec<_>>();
        positions_actual.sort();

        assert_eq!(
            positions_expected, positions_actual,
            "i = {:?}, text = {:?}, pattern = {:?}",
            i, text, pattern
        );
    }
}

#[test]
fn test_search_iter_matches_text_id() {
    let text_size = 1024;
    let alphabet_size = 8;
    let pattern_size_max = 128;

    let mut rng = StdRng::seed_from_u64(0);
    let text = testutil::build_text(|| rng.gen::<u8>() % alphabet_size, text_size);

    let fm_index = MultiTextFMIndexWithLocate::new(text.clone(), IdConverter::new::<u8>(), 0);

    for i in 0..1000 {
        let pattern_size = rng.gen::<usize>() % (pattern_size_max - 1) + 1;
        let pattern = (0..pattern_size)
            .map(|_| rng.gen::<u8>() % (alphabet_size - 1) + 1)
            .collect::<Vec<_>>();

        let mut text_ids_expected = Vec::new();
        let mut text_id = 0;
        for i in 0..=(text_size - pattern_size) {
            if text[i] == 0 {
                text_id += 1;
            }
            if text[i..i + pattern_size] == pattern {
                text_ids_expected.push(TextId::from(text_id));
            }
        }
        let mut text_ids_actual = fm_index
            .search(&pattern)
            .iter_matches()
            .map(|m| m.text_id())
            .collect::<Vec<_>>();
        text_ids_actual.sort();

        assert_eq!(
            text_ids_expected, text_ids_actual,
            "i = {:?}, text = {:?}, pattern = {:?}",
            i, text, pattern
        );
    }
}

fn generate_text_random(text_size: usize, alphabet_size: u8) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(0);

    (0..text_size)
        .map(|_| rng.gen::<u8>() % alphabet_size)
        .collect::<Vec<_>>()
}
