use fm_index::suffix_array::{IndexWithSA, SuffixOrderSampler};
use fm_index::{BackwardSearchIndex, FMIndex, RLFMIndex};

use criterion::{criterion_group, criterion_main};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};

mod common;

fn prepare_fmindex(
    len: usize,
    prob: f64,
    m: usize,
    l: usize,
) -> (impl BackwardSearchIndex<T = u8> + IndexWithSA, Vec<String>) {
    let (text, converter) = common::binary_text_set(len, prob);
    let patterns = common::binary_patterns(m);
    (
        FMIndex::new(text, converter, SuffixOrderSampler::new().level(l)),
        patterns,
    )
}

fn prepare_rlfmindex(
    len: usize,
    prob: f64,
    m: usize,
    l: usize,
) -> (impl BackwardSearchIndex<T = u8> + IndexWithSA, Vec<String>) {
    let (text, converter) = common::binary_text_set(len, prob);
    let patterns = common::binary_patterns(m);
    (
        RLFMIndex::new(text, converter, SuffixOrderSampler::new().level(l)),
        patterns,
    )
}

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("locate");
    let n = 50000;
    let m = 8;
    let prob = 0.5f64;
    group.throughput(Throughput::Elements(1 << m as u32));
    for l in [1, 2, 3].iter() {
        group.bench_with_input(BenchmarkId::new("FMIndex", l), l, |b, &l| {
            b.iter_batched(
                || prepare_fmindex(n, prob, m, l),
                |(index, patterns)| {
                    for pattern in patterns {
                        index.search_backward(pattern).locate();
                    }
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("RLFMIndex", l), l, |b, &l| {
            b.iter_batched(
                || prepare_rlfmindex(n, prob, m, l),
                |(index, patterns)| {
                    for pattern in patterns {
                        index.search_backward(pattern).locate();
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
