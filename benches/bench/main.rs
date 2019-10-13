mod construction;
mod count;
mod common;

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, construction::bench, count::bench);
criterion_main!(benches);
