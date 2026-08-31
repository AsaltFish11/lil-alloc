use criterion::{BenchmarkId, Criterion};
use std::hint::black_box;

use crate::common::{LARGE_COUNT, SMALL_COUNT, create_pool};

pub fn bench_single_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_allocation");

    // ============================================================
    // 1. 单个 i32：MemoryPool
    // ============================================================
    group.bench_function("pool_alloc_1_i32", |b| {
        let mut pool = create_pool();

        b.iter(|| {
            let p = pool
                .allocate::<i32>(SMALL_COUNT)
                .expect("pool allocation failed");

            unsafe {
                p.as_ptr().write(black_box(42));
            }

            unsafe {
                pool.deallocate(black_box(p.as_ptr()));
            }
        });
    });

    // ============================================================
    // 2. 单个 i32：Box
    // ============================================================
    group.bench_function("box_alloc_1_i32", |b| {
        b.iter(|| {
            let value = Box::new(black_box(42i32));
            black_box(value);
        });
    });

    // ============================================================
    // 3. 4096 个 i32：MemoryPool
    //
    // 注意：
    // 这里把 4096 个元素全部初始化。
    // 与 Box 的工作量保持一致。
    // ============================================================
    group.bench_function(BenchmarkId::new("pool_alloc_array_i32", LARGE_COUNT), |b| {
        let mut pool = create_pool();

        b.iter(|| {
            let p = pool
                .allocate::<i32>(LARGE_COUNT)
                .expect("pool allocation failed");

            unsafe {
                for i in 0..LARGE_COUNT {
                    p.as_ptr().add(i).write(42);
                }
            }

            // 防止整个 allocation 被优化掉。
            black_box(p.as_ptr());

            unsafe {
                pool.deallocate(p.as_ptr());
            }
        });
    });

    // ============================================================
    // 4. 4096 个 i32：Box
    // ============================================================
    group.bench_function(BenchmarkId::new("box_alloc_array_i32", LARGE_COUNT), |b| {
        b.iter(|| {
            let value = Box::new([42i32; LARGE_COUNT]);
            black_box(value);
        });
    });

    group.finish();
}
