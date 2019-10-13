use fm_index::suffix_array::NullSampler;
use fm_index::{FMIndex, RLFMIndex};

use criterion::{AxisScale, BatchSize, BenchmarkId, Criterion, PlotConfiguration};

use crate::common::binary_text_set;

pub fn bench(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("construction");
    group.plot_config(plot_config);
    for i in [1000usize, 10_000usize, 100_000usize].iter() {
        group.bench_with_input(BenchmarkId::new("FMIndex", i), i, |b, &i| {
            b.iter_batched(
                || binary_text_set(i, 0.5),
                |(text, converter)| {
                    FMIndex::new(text, converter, NullSampler::new());
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("RLFMIndex", i), i, |b, &i| {
            b.iter_batched(
                || binary_text_set(i, 0.5),
                |(text, converter)| {
                    RLFMIndex::new(text, converter, NullSampler::new());
                },
                BatchSize::SmallInput,
            )
        });
    }
}
