use fm_index::converter::RangeConverter;
use fm_index::{SearchIndexBuilder, SearchIndexWithLocate, SearchResult, SearchResultWithLocate};

fn main() {
    // Prepare a text string to search for patterns.
    let text = concat!(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
        "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.",
        "Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
    ).as_bytes().to_vec();

    // Converter converts each character into packed representation.
    // `' '` ~ `'~'` represents a range of ASCII printable characters.
    let converter = RangeConverter::new(b' ', b'~');

    // To perform locate queries, we need to use some storage.
    let index = SearchIndexBuilder::with_converter(converter)
        // the sampling level determines how much is retained in order to support `locate`
        // queries. `0` retains the full information, but we don't need the whole array
        // since we can interpolate missing elements in a suffix array from others. A sampler
        // will _sieve_ a suffix array for this purpose. If you don't need `locate` queries
        // you can save the memory by not setting a sampling level.
        .sampling_level(2)
        .build(text);

    // Search for a pattern string.
    let pattern = "dolor";
    let search = index.search(&pattern);

    // Count the number of occurrences.
    let n = search.count();
    assert_eq!(n, 4);

    // List the position of all occurrences.
    let positions = search.locate();
    assert_eq!(positions, vec![246, 12, 300, 103]);

    // Extract preceding characters from a search position.
    let i = 0;
    let mut prefix = search.iter_backward(i).take(16).collect::<Vec<u8>>();
    prefix.reverse();
    assert_eq!(prefix, b"Duis aute irure ".to_owned());

    // Extract succeeding characters from a search position.
    let i = 3;
    let postfix = search.iter_forward(i).take(20).collect::<Vec<u8>>();
    assert_eq!(postfix, b"dolore magna aliqua.".to_owned());

    // Search can be chained backward.
    let search_chained = search.search("et ");
    assert_eq!(search_chained.count(), 1);
}
