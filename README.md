# fm-index

This crate provides implementations of FM Index and its variants.

*FM Index*, originally proposed by Paolo Ferragina and Giovanni Manzini,
is a compressed full-text index which supports the following queries:

- `count`: Given a pattern string, counts the number of its occurrences.
- `locate`: Given a pattern string, lists the all position of its occurrences.
- `extract`: Given an integer, gets the character of the text at that position.

License: MIT
