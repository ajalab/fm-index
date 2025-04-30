<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

We added an experimental support for search indices of multiple texts (text pieces): `FMIndexMultiPieces` and `FMIndexMultiPiecesWithLocate`. Its APIs and internal implementations might be changed in the future.

### Changed

We made a large overhaul in the public APIs.

- Each FM-index variant has two structs: one supporting locate queries (e.g., `FMIndexWithLocate`) and the other doesn't (e.g., `FMIndex`).
- The constructors accept a text wrapped with a struct `Text`, which retains the maximum character along with the original text.
- The constructors require an explicit zero character (`\0`) at the end of the provided text. If they fail to build a search index, it returns `Result::Err`.

## [0.2.0] - 2024-12-21

### Features

We have changed to use [Vers](https://github.com/Cydhra/vers) for its
underlying rank/select data structures rather than Fids. It also uses the Vers
implementation of `WaveletMatrix` rather than the version custom one included
previously.

As a result of the port to Vers, the operations for count and locate accesses
are now 2 - 6 times faster than they were before in the included benchmarks.
The construction of the index is a little bit slower than in the
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

<!-- next-url -->
[Unreleased]: https://github.com/ajalab/fm-index/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ajalab/fm-index/compare/v0.1.2...v0.2.0
