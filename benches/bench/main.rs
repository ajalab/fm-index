mod common;
mod construction;
mod count;
mod locate;

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, construction::bench, count::bench, locate::bench);
criterion_main!(benches);
