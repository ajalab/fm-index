use fm_index::{FMIndex, RLFMIndex, SearchIndexBuilder};

use criterion::{criterion_group, criterion_main};
use criterion::{AxisScale, BatchSize, BenchmarkId, Criterion, PlotConfiguration};

mod common;

pub fn bench(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("construction");
    group.plot_config(plot_config);
    for n in [1000usize, 10_000usize, 100_000usize, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("FMIndex", n), n, |b, &n| {
            b.iter_batched(
                || common::binary_text_set(n, 0.5),
                |(text, converter)| {
                    SearchIndexBuilder::with_converter(converter)
                        .count_only()
                        .build(text)
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("RLFMIndex", n), n, |b, &n| {
            b.iter_batched(
                || common::binary_text_set(n, 0.5),
                |(text, converter)| {
                    SearchIndexBuilder::with_converter(converter)
                        .run_length_encoding()
                        .count_only()
                        .build(text)
                },
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
