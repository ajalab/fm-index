// tests that exercise the public API, especially the traits

use fm_index::{
    FMIndex, FMIndexWithLocate, HeapSize, RLFMIndex, RLFMIndexWithLocate, SearchIndex, Text,
};

fn len<T: SearchIndex<u8>>(index: &T) -> usize {
    index.len()
}

fn size<T: HeapSize>(t: &T) -> usize {
    t.heap_size()
}

#[test]
fn test_fm_index_backend_trait_fm_index_suffix_array() {
    let index = FMIndexWithLocate::new(&Text::new("text\0".as_bytes()), 2).unwrap();

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_fm_index_suffix_array() {
    let index = FMIndexWithLocate::new(&Text::new("text\0".as_bytes()), 2).unwrap();

    // any result will do for this test
    assert!(size(&index) > 0);
}

#[test]
fn test_fm_index_backend_trait_fm_index_count_only() {
    let index = FMIndex::new(&Text::new("text\0".as_bytes())).unwrap();

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_fm_index_count_only() {
    let index = FMIndex::new(&Text::new("text\0".as_bytes())).unwrap();

    // any result will do for this test
    assert!(size(&index) > 0);
}

#[test]
fn test_fm_index_backend_trait_rlfm_index_suffix_array() {
    let index = RLFMIndexWithLocate::new(&Text::new("text\0".as_bytes()), 2).unwrap();

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_rlfm_index_suffix_array() {
    let index = RLFMIndexWithLocate::new(&Text::new("text\0".as_bytes()), 2).unwrap();

    // any result will do for this test
    assert!(size(&index) > 0);
}

#[test]
fn test_fm_index_backend_trait_rlfm_index_count_only() {
    let index = RLFMIndex::new(&Text::new("text\0".as_bytes())).unwrap();

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_rlfm_index_count_only() {
    let index = RLFMIndex::new(&Text::new("text\0".as_bytes())).unwrap();

    // any result will do for this test
    assert!(size(&index) > 0);
}
