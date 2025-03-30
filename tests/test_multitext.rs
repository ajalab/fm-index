mod testutil;
use fm_index::converter::IdConverter;
use fm_index::{MatchWithLocate, MatchWithTextId, MultiTextFMIndexWithLocate, Search, TextId};
use testutil::TestRunner;

#[test]
fn test_search_count() {
    let text_size = 1024;

    TestRunner {
        texts: 100,
        patterns: 100,
        text_size,
        alphabet_size: 8,
        level_max: 3,
        pattern_size_max: 10,
    }
    .run(
        |text, level| {
            MultiTextFMIndexWithLocate::new(text.clone(), IdConverter::new::<u8>(), level)
        },
        |fm_index, text, pattern| {
            let mut count_expected = 0;
            for i in 0..=(text_size - pattern.len()) {
                if &text[i..i + pattern.len()] == pattern {
                    count_expected += 1;
                }
            }
            let count_actual = fm_index.search(pattern).count();

            assert_eq!(
                count_expected, count_actual,
                "text = {:?}, pattern = {:?}",
                text, pattern
            );
        },
    );
}

#[test]
fn test_search_locate() {
    let text_size = 1024;

    TestRunner {
        texts: 100,
        patterns: 100,
        text_size,
        alphabet_size: 8,
        level_max: 3,
        pattern_size_max: 10,
    }
    .run(
        |text, level| {
            MultiTextFMIndexWithLocate::new(text.clone(), IdConverter::new::<u8>(), level)
        },
        |fm_index, text, pattern| {
            let mut positions_expected = Vec::new();
            for i in 0..=(text_size - pattern.len()) {
                if &text[i..i + pattern.len()] == pattern {
                    positions_expected.push(i as u64);
                }
            }
            let mut positions_actual = fm_index.search(&pattern).locate();
            positions_actual.sort();

            assert_eq!(
                positions_expected, positions_actual,
                "text = {:?}, pattern = {:?}",
                text, pattern
            );
        },
    );
}

#[test]
fn test_search_iter_matches_locate() {
    let text_size = 1024;

    TestRunner {
        texts: 100,
        patterns: 100,
        text_size,
        alphabet_size: 8,
        level_max: 3,
        pattern_size_max: 10,
    }
    .run(
        |text, level| {
            MultiTextFMIndexWithLocate::new(text.clone(), IdConverter::new::<u8>(), level)
        },
        |fm_index, text, pattern| {
            let mut positions_expected = Vec::new();
            for i in 0..=(text_size - pattern.len()) {
                if &text[i..i + pattern.len()] == pattern {
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
                "text = {:?}, pattern = {:?}",
                text, pattern
            );
        },
    );
}

#[test]
fn test_search_iter_matches_text_id() {
    let text_size = 1024;

    TestRunner {
        texts: 100,
        patterns: 100,
        text_size,
        alphabet_size: 8,
        level_max: 3,
        pattern_size_max: 10,
    }
    .run(
        |text, level| {
            MultiTextFMIndexWithLocate::new(text.clone(), IdConverter::new::<u8>(), level)
        },
        |fm_index, text, pattern| {
            let mut text_ids_expected = Vec::new();
            let mut text_id = 0;
            for i in 0..=(text_size - pattern.len()) {
                if text[i] == 0 {
                    text_id += 1;
                }
                if &text[i..i + pattern.len()] == pattern {
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
                "text = {:?}, pattern = {:?}",
                text, pattern
            );
        },
    );
}
