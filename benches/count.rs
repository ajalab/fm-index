use fm_index::suffix_array::NullSampler;
use fm_index::{BackwardSearchIndex, FMIndex, RLFMIndex};

use criterion::{criterion_group, criterion_main};
use criterion::{AxisScale, BatchSize, BenchmarkId, Criterion, PlotConfiguration, Throughput};

mod common;

fn prepare_fmindex(
    len: usize,
    prob: f64,
    m: usize,
) -> (impl BackwardSearchIndex<T = u8>, Vec<String>) {
    let (text, converter) = common::binary_text_set(len, prob);
    let patterns = common::binary_patterns(m);
    (FMIndex::new(text, converter, NullSampler::new()), patterns)
}

fn prepare_rlfmindex(
    len: usize,
    prob: f64,
    m: usize,
) -> (impl BackwardSearchIndex<T = u8>, Vec<String>) {
    let (text, converter) = common::binary_text_set(len, prob);
    let patterns = common::binary_patterns(m);
    (
        RLFMIndex::new(text, converter, NullSampler::new()),
        patterns,
    )
}

pub fn bench(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("count");
    let n = 50000;
    let m = 8;
    group.plot_config(plot_config);
    group.throughput(Throughput::Elements(1 << m as u32));
    for prob in [0.5, 0.05, 0.005].iter() {
        group.bench_with_input(BenchmarkId::new("FMIndex", prob), prob, |b, &prob| {
            b.iter_batched(
                || prepare_fmindex(n, prob, m),
                |(index, patterns)| {
                    for pattern in patterns {
                        index.search_backward(pattern).count();
                    }
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("RLFMIndex", prob), prob, |b, &prob| {
            b.iter_batched(
                || prepare_rlfmindex(n, prob, m),
                |(index, patterns)| {
                    for pattern in patterns {
                        index.search_backward(pattern).count();
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
