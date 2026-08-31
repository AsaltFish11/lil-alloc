use criterion::Criterion;
use std::hint::black_box;

use crate::common::{POOL_SIZE, create_pool};

pub fn bench_pool_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_reuse");

    // ============================================================
    // 1. 创建 MemoryPool
    // ============================================================
    group.bench_function("pool_creation", |b| {
        b.iter(|| {
            black_box(create_pool());
        });
    });

    // ============================================================
    // 2. MemoryPool 填满
    // ============================================================
    group.bench_function("pool_full_alloc_free", |b| {
        b.iter(|| {
            let mut pool = create_pool();

            // 提前预留容量，避免 Vec 扩容成为主要干扰。
            let estimated_count = POOL_SIZE / 1024;

            let mut ptrs = Vec::with_capacity(estimated_count);

            while let Some(p) = pool.allocate::<u8>(1024) {
                ptrs.push(p);
            }

            for p in ptrs.into_iter().rev() {
                unsafe {
                    pool.deallocate(p.as_ptr());
                }
            }
        });
    });

    group.finish();
}
