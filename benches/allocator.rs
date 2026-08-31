mod bulk;
mod common;
mod dynamic;
mod fragmentation;
mod reuse;
mod single;

use criterion::criterion_group;
use criterion::criterion_main;

criterion_group!(
    benches,
    single::bench_single_allocation,
    bulk::bench_bulk_allocation,
    dynamic::bench_dynamic_allocation,
    reuse::bench_pool_reuse,
    fragmentation::bench_fragmentation,
);

criterion_main!(benches);
