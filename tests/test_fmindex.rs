mod testutil;
use fm_index::{FMIndexWithLocate, MatchWithLocate, Search, Text};
use testutil::TestRunner;

#[test]
fn test_small_search_count() {
    let text = Text::new("a\0".as_bytes());
    let fm_index = FMIndexWithLocate::new(&text, 2).unwrap();

    assert_eq!(1, fm_index.search("a").count());
}

#[test]
fn test_small_search_locate() {
    let text = Text::new("a\0".as_bytes());
    let fm_index = FMIndexWithLocate::new(&text, 2).unwrap();

    let positions = fm_index
        .search("a")
        .iter_matches()
        .map(|m| m.locate())
        .collect::<Vec<_>>();
    assert_eq!(vec![0], positions);
}

#[test]
fn test_random_search_count() {
    let text_size_max = 1024;

    TestRunner {
        texts: 100,
        patterns: 100,
        text_size_max,
        alphabet_size: 8,
        level_max: 3,
        pattern_size_max: 10,
        multi_pieces: false,
    }
    .run(FMIndexWithLocate::new, |fm_index, text, pattern| {
        let naive_index = testutil::NaiveSearchIndex::new(text.text());
        let matches_expected = naive_index.search(pattern);

        let count_expected = matches_expected.len();
        let count_actual = fm_index.search(pattern).count();
        assert_eq!(
            count_expected,
            count_actual,
            "text = {:?}, pattern = {:?}",
            text.text(),
            pattern
        );
    });
}
#[test]
fn test_random_search_locate() {
    let text_size_max = 100;

    TestRunner {
        texts: 100,
        patterns: 100,
        text_size_max,
        alphabet_size: 8,
        level_max: 3,
        pattern_size_max: 10,
        multi_pieces: false,
    }
    .run(FMIndexWithLocate::new, |fm_index, text, pattern| {
        let naive_index = testutil::NaiveSearchIndex::new(text.text());
        let matches_expected = naive_index.search(pattern);

        let positions_expected = matches_expected
            .iter()
            .map(|m| m.position)
            .collect::<Vec<_>>();
        let mut positions_actual = fm_index
            .search(pattern)
            .iter_matches()
            .map(|m| m.locate())
            .collect::<Vec<_>>();
        positions_actual.sort();
        assert_eq!(
            positions_expected,
            positions_actual,
            "text = {:?}, pattern = {:?}",
            text.text(),
            pattern
        );
    });
}
