// tests that exercise the public API, especially the traits

use fm_index::{
    converter::IdConverter, FMIndex, FMIndexWithLocate, HeapSize, RLFMIndex, RLFMIndexWithLocate,
    SearchIndex,
};

fn len<T: SearchIndex<u8>>(index: &T) -> u64 {
    index.len()
}

fn size<T: HeapSize>(t: &T) -> usize {
    t.heap_size()
}

#[test]
fn test_fm_index_backend_trait_fm_index_suffix_array() {
    let index = FMIndexWithLocate::new("text".as_bytes().to_vec(), IdConverter::new::<u8>(), 2);

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_fm_index_suffix_array() {
    let index = FMIndexWithLocate::new("text".as_bytes().to_vec(), IdConverter::new::<u8>(), 2);

    // any result will do for this test
    assert!(size(&index) > 0);
}

#[test]
fn test_fm_index_backend_trait_fm_index_count_only() {
    let index = FMIndex::new("text".as_bytes().to_vec(), IdConverter::new::<u8>());

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_fm_index_count_only() {
    let index = FMIndex::new("text".as_bytes().to_vec(), IdConverter::new::<u8>());

    // any result will do for this test
    assert!(size(&index) > 0);
}

#[test]
fn test_fm_index_backend_trait_rlfm_index_suffix_array() {
    let index = RLFMIndexWithLocate::new("text".as_bytes().to_vec(), IdConverter::new::<u8>(), 2);

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_rlfm_index_suffix_array() {
    let index = RLFMIndexWithLocate::new("text".as_bytes().to_vec(), IdConverter::new::<u8>(), 2);

    // any result will do for this test
    assert!(size(&index) > 0);
}

#[test]
fn test_fm_index_backend_trait_rlfm_index_count_only() {
    let index = RLFMIndex::new("text".as_bytes().to_vec(), IdConverter::new::<u8>());

    // any result will do for this test
    assert_eq!(len(&index), 5);
}

#[test]
fn test_heap_size_trait_rlfm_index_count_only() {
    let index = RLFMIndex::new("text".as_bytes().to_vec(), IdConverter::new::<u8>());

    // any result will do for this test
    assert!(size(&index) > 0);
}
