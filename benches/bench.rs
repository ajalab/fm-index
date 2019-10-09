use fm_index::converter::RangeConverter;
use fm_index::suffix_array::RegularSampler;
use fm_index::FMIndex;

use criterion::BatchSize;
use criterion::{
    criterion_group, criterion_main, AxisScale, Criterion, ParameterizedBenchmark,
    PlotConfiguration,
};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn prepare(len: usize) -> Vec<u8> {
    let prob = 0.5;
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let mut text = (0..len)
        .map(|_| if rng.gen_bool(prob) { b'a' } else { b'b' })
        .collect::<Vec<_>>();
    text.push(0);
    text
}

fn construct_fm_index(text: Vec<u8>) {
    FMIndex::new(
        text,
        RangeConverter::new(b'a', b'b'),
        RegularSampler::new().level(2),
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    c.bench(
        "fm_index",
        ParameterizedBenchmark::new(
            "change_text_size",
            |b, i| b.iter_batched(|| prepare(*i), construct_fm_index, BatchSize::SmallInput),
            vec![10_000usize, 1_000_000usize, 100_000_000usize],
        )
        .plot_config(plot_config),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
