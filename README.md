# fm-index

This crate provides implementations of FM Index and its variants.

*FM Index*, originally proposed by Paolo Ferragina and Giovanni Manzini [1],
is a compressed full-text index which supports the following queries:

- `count`: Given a pattern string, counts the number of its occurrences.
- `locate`: Given a pattern string, lists the all position of its occurrences.
- `extract`: Given an integer, gets the character of the text at that position.

`fm-index` crate does not support the third query (extracting a character from arbitrary position) now.
Instead, it provides backward/forward iterators that return the text characters starting from a search result.

## Usage

Add this to your `Cargo.toml`.

```toml
[dependencies]
fm-index = "0.1"
```

## Example
```rust
use fm_index::converter::RangeConverter;
use fm_index::suffix_array::RegularSampler;
use fm_index::{BackwardSearchIndex, FMIndex};

// Prepare a text string to search for patterns. Make sure it should contain \0 at the end.
let text = concat!(
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
    "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.",
    "Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\0",
).as_bytes().to_vec();

// Converter converts each character into packed representation.
// `' '` ~ `'~'` represents a range of ASCII printable characters.
let converter = RangeConverter::new(b' ', b'~');

// To perform locate queries, we need to retain suffix array generated in the construction phase.
// However, we don't need the whole array since we can interpolate missing elements in a suffix array from others.
// A sampler will _sieve_ a suffix array for this purpose.
// You can also use `NullSampler` if you don't perform location queries (disabled in type-level).
let sampler = RegularSampler::new().level(2);
let index = FMIndex::new(text, converter, sampler);

// Search for a pattern string.
let pattern = "dolor";
let search = index.search_backward(pattern);

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
let search_chained = search.search_backward("et ");
assert_eq!(search_chained.count(), 1);
```

## Reference

[1] Paolo Ferragina and Giovanni Manzini (2000). "Opportunistic Data Structures with Applications". Proceedings of the 41st Annual Symposium on Foundations of Computer Science. p.390.

## License

MIT

License: MIT
