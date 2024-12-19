# fm-index-vers

[![Crate](https://img.shields.io/crates/v/fm-index-vers.svg)](https://crates.io/crates/fm-index-vers)
[![Doc](https://docs.rs/fm-index-vers/badge.svg)](https://docs.rs/fm-index-vers)

This crate provides implementations of
[FM-Index](https://en.wikipedia.org/wiki/FM-index) and its variants.

FM-Index, originally proposed by Paolo Ferragina and Giovanni Manzini [1],
is a compressed full-text index which supports the following queries:

- `count`: Given a pattern string, counts the number of its occurrences.
- `locate`: Given a pattern string, lists the all positions of its occurrences.
- `extract`: Given an integer, gets the character of the text at that position.

The `fm-index-vers` crate does not support the third query (extracting a
character from arbitrary position). Instead, it provides backward/forward
iterators that return the text characters starting from a search result.

## Background

This is a port of the [original
implementation](https://github.com/ajalab/fm-index) by [Koki
Kato](https://github.com/ajalab) to use [Vers](https://github.com/Cydhra/vers)
for its underlying rank/select data structures. It also uses the Vers
implementation of `WaveletMatrix` rather than the version custom built for
`fm-index`.

As a result of the port to Vers, the operations for count and locate accesses
are now 2 - 6 times faster than they were before in the included benchmarks.
The construction of the index is currently a little bit slower than in the
original.

```
group                             vers port                                original
-----                             ---------                                --------
construction/FMIndex/1000         1.03     45.4±0.93µs        ? ?/sec      1.00     44.2±1.65µs        ? ?/sec
construction/FMIndex/10000        1.02    647.8±3.15µs        ? ?/sec      1.00    635.5±8.88µs        ? ?/sec
construction/FMIndex/100000       1.10      7.8±0.07ms        ? ?/sec      1.00      7.1±0.09ms        ? ?/sec
construction/FMIndex/1000000      1.33    101.8±4.82ms        ? ?/sec      1.00     76.7±2.58ms        ? ?/sec
construction/RLFMIndex/1000       1.09     52.7±2.04µs        ? ?/sec      1.00     48.2±0.78µs        ? ?/sec
construction/RLFMIndex/10000      1.01    686.2±2.54µs        ? ?/sec      1.00    680.5±3.61µs        ? ?/sec
construction/RLFMIndex/100000     1.03      8.0±0.12ms        ? ?/sec      1.00      7.7±0.03ms        ? ?/sec
construction/RLFMIndex/1000000    1.09    101.8±1.87ms        ? ?/sec      1.00     93.7±4.28ms        ? ?/sec
count/FMIndex/0.005               1.00     67.6±1.55µs  3.6 MElem/sec      2.46    166.1±2.72µs 1505.5 KElem/sec
count/FMIndex/0.05                1.00    118.9±4.70µs  2.1 MElem/sec      3.04    360.9±7.14µs 692.7 KElem/sec
count/FMIndex/0.5                 1.00    116.1±2.15µs  2.1 MElem/sec      3.46    402.2±1.76µs 621.5 KElem/sec
count/RLFMIndex/0.005             1.00    232.5±4.62µs 1075.2 KElem/sec    1.91    444.2±6.78µs 562.8 KElem/sec
count/RLFMIndex/0.05              1.00    426.4±9.77µs 586.3 KElem/sec     2.51  1069.7±10.15µs 233.7 KElem/sec
count/RLFMIndex/0.5               1.00   494.0±11.52µs 506.1 KElem/sec     2.60   1285.7±8.28µs 194.4 KElem/sec
locate/FMIndex/1                  1.00      3.2±0.05ms 79.1 KElem/sec      4.49     14.2±0.04ms 17.6 KElem/sec
locate/FMIndex/2                  1.00      8.3±0.02ms 30.1 KElem/sec      4.99     41.4±0.57ms  6.0 KElem/sec
locate/FMIndex/3                  1.00     18.2±0.05ms 13.8 KElem/sec      5.39     97.9±0.35ms  2.6 KElem/sec
locate/RLFMIndex/1                1.00      8.9±0.02ms 28.0 KElem/sec      2.80     25.0±0.08ms 10.0 KElem/sec
locate/RLFMIndex/2                1.00     24.9±0.12ms 10.0 KElem/sec      3.02     75.3±1.13ms  3.3 KElem/sec
locate/RLFMIndex/3                1.00     57.7±0.29ms  4.3 KElem/sec      3.25    187.8±2.91ms  1363 Elem/sec
```

If compiled using `-C target-cpu=native` (so that Vers can make use of additional optimizations) it's faster yet:

```
group                             vers port (native)                       original
-----                             ------------------                       --------
construction/FMIndex/1000         1.05     46.5±1.06µs        ? ?/sec      1.00     44.2±1.65µs        ? ?/sec
construction/FMIndex/10000        1.03   657.6±11.59µs        ? ?/sec      1.00    635.5±8.88µs        ? ?/sec
construction/FMIndex/100000       1.14      8.0±0.09ms        ? ?/sec      1.00      7.1±0.09ms        ? ?/sec
construction/FMIndex/1000000      1.27     97.0±1.08ms        ? ?/sec      1.00     76.7±2.58ms        ? ?/sec
construction/RLFMIndex/1000       1.12     54.1±1.16µs        ? ?/sec      1.00     48.2±0.78µs        ? ?/sec
construction/RLFMIndex/10000      1.04    708.1±3.77µs        ? ?/sec      1.00    680.5±3.61µs        ? ?/sec
construction/RLFMIndex/100000     1.05      8.1±0.11ms        ? ?/sec      1.00      7.7±0.03ms        ? ?/sec
construction/RLFMIndex/1000000    1.06     99.3±1.38ms        ? ?/sec      1.00     93.7±4.28ms        ? ?/sec
count/FMIndex/0.005               1.00     47.1±1.12µs  5.2 MElem/sec      3.53    166.1±2.72µs 1505.5 KElem/sec
count/FMIndex/0.05                1.00     75.6±0.53µs  3.2 MElem/sec      4.78    360.9±7.14µs 692.7 KElem/sec
count/FMIndex/0.5                 1.00     86.0±2.29µs  2.8 MElem/sec      4.68    402.2±1.76µs 621.5 KElem/sec
count/RLFMIndex/0.005             1.00    115.3±0.96µs  2.1 MElem/sec      3.85    444.2±6.78µs 562.8 KElem/sec
count/RLFMIndex/0.05              1.00    203.8±1.98µs 1226.9 KElem/sec    5.25  1069.7±10.15µs 233.7 KElem/sec
count/RLFMIndex/0.5               1.00    252.8±3.94µs 988.8 KElem/sec     5.09   1285.7±8.28µs 194.4 KElem/sec
locate/FMIndex/1                  1.00      2.7±0.02ms 93.6 KElem/sec      5.31     14.2±0.04ms 17.6 KElem/sec
locate/FMIndex/2                  1.00      7.1±0.04ms 35.2 KElem/sec      5.84     41.4±0.57ms  6.0 KElem/sec
locate/FMIndex/3                  1.00     15.6±0.08ms 16.0 KElem/sec      6.26     97.9±0.35ms  2.6 KElem/sec
locate/RLFMIndex/1                1.00      5.2±0.02ms 48.5 KElem/sec      4.84     25.0±0.08ms 10.0 KElem/sec
locate/RLFMIndex/2                1.00     14.1±0.06ms 17.7 KElem/sec      5.33     75.3±1.13ms  3.3 KElem/sec
locate/RLFMIndex/3                1.00     31.9±0.09ms  7.8 KElem/sec      5.89    187.8±2.91ms  1363 Elem/sec
```

## Usage

Add this to your `Cargo.toml`.

```toml
[dependencies]
fm-index-vers = "0.1"
```

## Example
```rust
use fm_index_vers::converter::RangeConverter;
use fm_index_vers::suffix_array::SuffixOrderSampler;
use fm_index_vers::{BackwardSearchIndex, FMIndex};

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

// To perform locate queries, we need to retain suffix array generated in the construction phase.
// However, we don't need the whole array since we can interpolate missing elements in a suffix array from others.
// A sampler will _sieve_ a suffix array for this purpose.
// You can also use `NullSampler` if you don't perform location queries (disabled in type-level).
let sampler = SuffixOrderSampler::new().level(2);
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

## Implementations

### FM-Index

The implementation is based on [1]. The index is constructed with a suffix
array generated by SA-IS [3] in _O(n)_ time, where _n_ is the size of a text
string.

Basically it consists of

- a Burrows-Wheeler transform (BWT) of the text string represented with
  _wavelet matrix_ [4]
- an array of size _O(σ)_ (_σ_: number of characters) which stores the number
  of characters smaller than a given character
- a (sampled) suffix array

### Run-Length FM-Index

Based on [2]. The index is constructed with a suffix array generated by SA-IS [3].

It consists of

- a wavelet matrix that stores the run heads of BWT of the text string
- a succinct bit vector which stores the run lengths of BWT of the text string
- a succinct bit vector which stores the run lengths of BWT of the text string
  sorted in alphabetical order of corresponding run heads
- an array of size _O(σ)_ (_σ_: number of characters) which stores the number
  of characters smaller than a given character in run heads

## Reference

[1] Ferragina, P., & Manzini, G. (2000). Opportunistic data structures with
applications. Annual Symposium on Foundations of Computer Science -
Proceedings, 390–398. https://doi.org/10.1109/sfcs.2000.892127

[2] Mäkinen, V., & Navarro, G. (2005). Succinct suffix arrays based on
run-length encoding. In Lecture Notes in Computer Science (Vol. 3537).
https://doi.org/10.1007/11496656_5

[3] Ge Nong, Sen Zhang, & Wai Hong Chan. (2010). Two Efficient Algorithms for
Linear Time Suffix Array Construction. IEEE Transactions on Computers, 60(10),
1471–1484. https://doi.org/10.1109/tc.2010.188

[4] Claude F., Navarro G. (2012). The Wavelet Matrix. In: Calderón-Benavides
L., González-Caro C., Chávez E., Ziviani N. (eds) String Processing and
Information Retrieval. SPIRE 2012. https://doi.org/10.1007/978-3-642-34109-0_18
