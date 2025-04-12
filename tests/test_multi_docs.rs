mod testutil;
use fm_index::{FMIndexMultiDocsWithLocate, MatchWithLocate, MatchWithDocId, Search};
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
        multi_text: true,
    }
    .run(
        FMIndexMultiDocsWithLocate::new,
        |fm_index, text, pattern| {
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
        multi_text: true,
    }
    .run(
        FMIndexMultiDocsWithLocate::new,
        |fm_index, text, pattern| {
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
        },
    );
}

#[test]
fn test_search_doc_id() {
    let text_size = 1024;

    TestRunner {
        texts: 100,
        patterns: 100,
        text_size,
        alphabet_size: 8,
        level_max: 3,
        pattern_size_max: 10,
        multi_text: true,
    }
    .run(
        FMIndexMultiDocsWithLocate::new,
        |fm_index, text, pattern| {
            let naive_index = testutil::NaiveSearchIndex::new(text.text());
            let matches_expected = naive_index.search(pattern);

            let doc_ids_expected = matches_expected
                .iter()
                .map(|m| m.doc_id)
                .collect::<Vec<_>>();
            let mut doc_ids_actual = fm_index
                .search(pattern)
                .iter_matches()
                .map(|m| m.doc_id())
                .collect::<Vec<_>>();
            doc_ids_actual.sort();
            assert_eq!(
                doc_ids_expected,
                doc_ids_actual,
                "text = {:?}, pattern = {:?}",
                text.text(),
                pattern
            );
        },
    );
}

#[test]
fn test_search_prefix_doc_id() {
    let text_size = 1024;

    TestRunner {
        texts: 100,
        patterns: 100,
        text_size,
        alphabet_size: 8,
        level_max: 3,
        pattern_size_max: 10,
        multi_text: true,
    }
    .run(
        FMIndexMultiDocsWithLocate::new,
        |fm_index, text, pattern| {
            let naive_index = testutil::NaiveSearchIndex::new(text.text());
            let matches_expected = naive_index.search_prefix(pattern);

            let doc_ids_expected = matches_expected
                .iter()
                .map(|m| m.doc_id)
                .collect::<Vec<_>>();
            let mut doc_ids_actual = fm_index
                .search_prefix(pattern)
                .iter_matches()
                .map(|m| m.doc_id())
                .collect::<Vec<_>>();
            doc_ids_actual.sort();
            assert_eq!(
                doc_ids_expected,
                doc_ids_actual,
                "text = {:?}, pattern = {:?}",
                text.text(),
                pattern
            );
        },
    );
}

#[test]
fn test_search_suffix_doc_id() {
    let text_size = 1024;

    TestRunner {
        texts: 100,
        patterns: 100,
        text_size,
        alphabet_size: 8,
        level_max: 3,
        pattern_size_max: 10,
        multi_text: true,
    }
    .run(
        FMIndexMultiDocsWithLocate::new,
        |fm_index, text, pattern| {
            let naive_index = testutil::NaiveSearchIndex::new(text.text());
            let matches_expected = naive_index.search_suffix(pattern);

            let doc_ids_expected = matches_expected
                .iter()
                .map(|m| m.doc_id)
                .collect::<Vec<_>>();
            let mut doc_ids_actual = fm_index
                .search_suffix(pattern)
                .iter_matches()
                .map(|m| m.doc_id())
                .collect::<Vec<_>>();
            doc_ids_actual.sort();
            assert_eq!(
                doc_ids_expected,
                doc_ids_actual,
                "text = {:?}, pattern = {:?}",
                text.text(),
                pattern
            );
        },
    );
}

#[test]
fn test_search_exact_doc_id() {
    let text_size = 1024;

    TestRunner {
        texts: 100,
        patterns: 100,
        text_size,
        alphabet_size: 8,
        level_max: 3,
        pattern_size_max: 10,
        multi_text: true,
    }
    .run(
        FMIndexMultiDocsWithLocate::new,
        |fm_index, text, pattern| {
            let naive_index = testutil::NaiveSearchIndex::new(text.text());
            let matches_expected = naive_index.search_exact(pattern);

            let doc_ids_expected = matches_expected
                .iter()
                .map(|m| m.doc_id)
                .collect::<Vec<_>>();
            let mut doc_ids_actual = fm_index
                .search_exact(pattern)
                .iter_matches()
                .map(|m| m.doc_id())
                .collect::<Vec<_>>();
            doc_ids_actual.sort();
            assert_eq!(
                doc_ids_expected,
                doc_ids_actual,
                "text = {:?}, pattern = {:?}",
                text.text(),
                pattern
            );
        },
    );
}
