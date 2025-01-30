// tests that exercise the public API, especially the traits

use fm_index::{HeapSize, SearchIndex};

fn len<T: SearchIndex<u8>>(index: &T) -> u64 {
    index.len()
}

fn size<T: HeapSize>(t: &T) -> usize {
    t.size()
}

#[test]
fn test_fm_index_backend_trait_fm_index_suffix_array() {
    let builder = fm_index::SearchIndexBuilder::new();
    let text = "text";

    let index = builder.build(text.as_bytes().to_vec());

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_fm_index_suffix_array() {
    let builder = fm_index::SearchIndexBuilder::new();
    let text = "text";

    let index = builder.build(text.as_bytes().to_vec());

    // any result will do for this test
    assert!(size(&index) > 0);
}

#[test]
fn test_fm_index_backend_trait_fm_index_count_only() {
    let builder = fm_index::SearchIndexBuilder::new().count_only();
    let text = "text";

    let index = builder.build(text.as_bytes().to_vec());

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_fm_index_count_only() {
    let builder = fm_index::SearchIndexBuilder::new().count_only();
    let text = "text";

    let index = builder.build(text.as_bytes().to_vec());

    // any result will do for this test
    assert!(size(&index) > 0);
}

#[test]
fn test_fm_index_backend_trait_rlfm_index_suffix_array() {
    let builder = fm_index::SearchIndexBuilder::new().run_length_encoding();
    let text = "text";

    let index = builder.build(text.as_bytes().to_vec());

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_rlfm_index_suffix_array() {
    let builder = fm_index::SearchIndexBuilder::new().run_length_encoding();
    let text = "text";

    let index = builder.build(text.as_bytes().to_vec());

    // any result will do for this test
    assert!(size(&index) > 0);
}

#[test]
fn test_fm_index_backend_trait_rlfm_index_count_only() {
    let builder = fm_index::SearchIndexBuilder::new()
        .count_only()
        .run_length_encoding();
    let text = "text";

    let index = builder.build(text.as_bytes().to_vec());

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_rlfm_index_count_only() {
    let builder = fm_index::SearchIndexBuilder::new()
        .count_only()
        .run_length_encoding();
    let text = "text";

    let index = builder.build(text.as_bytes().to_vec());

    // any result will do for this test
    assert!(size(&index) > 0);
}
