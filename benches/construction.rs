use criterion::{criterion_group, criterion_main};
use criterion::{AxisScale, BatchSize, BenchmarkId, Criterion, PlotConfiguration};
use fm_index::{FMIndex, RLFMIndex};

mod common;

pub fn bench(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("construction");
    group.plot_config(plot_config);
    for n in [1000usize, 10_000usize, 100_000usize, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("FMIndex", n), n, |b, &n| {
            b.iter_batched(
                || common::binary_text_set(n, 0.5),
                |(text, converter)| FMIndex::new(text, converter),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("RLFMIndex", n), n, |b, &n| {
            b.iter_batched(
                || common::binary_text_set(n, 0.5),
                |(text, converter)| RLFMIndex::new(text, converter),
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
