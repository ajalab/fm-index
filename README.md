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

## Reference

[1] Paolo Ferragina and Giovanni Manzini (2000). "Opportunistic Data Structures with Applications". Proceedings of the 41st Annual Symposium on Foundations of Computer Science. p.390.

## License

MIT

License: MIT
