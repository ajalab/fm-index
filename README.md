# fm-index

[![Crate](https://img.shields.io/crates/v/fm-index.svg)](https://crates.io/crates/fm-index)
[![Doc](https://docs.rs/fm-index/badge.svg)](https://docs.rs/fm-index)

This crate provides implementations of
[FM-Index](https://en.wikipedia.org/wiki/FM-index) and its variants.

FM-Index, originally proposed by Paolo Ferragina and Giovanni Manzini [^1],
is a compressed full-text index which supports the following queries:

- `count`: Given a pattern string, counts the number of its occurrences.
- `locate`: Given a pattern string, lists the all positions of its occurrences.
- `extract`: Given an integer, gets the character of the text at that position.

The `fm-index` crate does not support the third query (extracting a
character from arbitrary position). Instead, it provides backward/forward
iterators that return the text characters starting from a search result.

## Usage

Add this to your `Cargo.toml`.

```toml
[dependencies]
fm-index = "0.2"
```

## Example

```rust
use fm_index::{FMIndexWithLocate, Match, MatchWithLocate, Search, Text};

// Prepare a text string to search for patterns.
let text = concat!(
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
    "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.",
    "Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
    "\0",
).as_bytes();
let text = Text::new(text);

// The sampling level determines how much is retained in order to support `locate`
// queries. `0` retains the full information, but we don't need the whole array
// since we can interpolate missing elements in a suffix array from others. A sampler
// will _sieve_ a suffix array for this purpose. If you don't need `locate` queries
// you can save the memory by not setting a sampling level.
let index = FMIndexWithLocate::new(&text, 2).unwrap();

// Search for a pattern string.
let pattern = "dolor";
let search = index.search(pattern);

// Count the number of occurrences.
let n = search.count();
assert_eq!(n, 4);

// List the position of all occurrences.
let positions = search
    .iter_matches()
    .map(|m| m.locate())
    .collect::<Vec<_>>();
assert_eq!(positions, vec![246, 12, 300, 103]);

// Extract preceding characters from a search position.
let mut prefix = search
    .iter_matches()
    .next()
    .unwrap()
    .iter_chars_backward()
    .take(16)
    .collect::<Vec<u8>>();
prefix.reverse();
assert_eq!(prefix, b"Duis aute irure ".to_owned());

// Extract succeeding characters from a search position.
let postfix = search
    .iter_matches()
    .nth(3)
    .unwrap()
    .iter_chars_forward()
    .take(20)
    .collect::<Vec<u8>>();
assert_eq!(postfix, b"dolore magna aliqua.".to_owned());
```

See [`examples/`](examples/) directory for more examples.

## Implementations

See the [crate document](https://docs.rs/fm-index/latest/fm_index/) for details about the implementation.
