use criterion::{BenchmarkId, Criterion};
use std::hint::black_box;

use crate::common::{BULK_COUNT, LARGE_COUNT, create_pool};

pub fn bench_bulk_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_allocation");

    // ============================================================
    // 1. 小对象反复 allocate/free
    // ============================================================
    group.bench_function(BenchmarkId::new("pool_alloc_free", BULK_COUNT), |b| {
        let mut pool = create_pool();

        b.iter(|| {
            for _ in 0..BULK_COUNT {
                let p = pool.allocate::<i32>(1).expect("pool allocation failed");

                unsafe {
                    p.as_ptr().write(42);
                    pool.deallocate(p.as_ptr());
                }
            }
        });
    });

    group.bench_function(BenchmarkId::new("box_alloc_free", BULK_COUNT), |b| {
        b.iter(|| {
            for _ in 0..BULK_COUNT {
                let value = Box::new(black_box(42i32));
                black_box(value);
            }
        });
    });

    // ============================================================
    // 2. 大对象反复 allocate/free
    // ============================================================
    group.bench_function(
        BenchmarkId::new(
            "pool_large_alloc_free",
            format!("{BULK_COUNT}x{LARGE_COUNT}"),
        ),
        |b| {
            let mut pool = create_pool();

            b.iter(|| {
                for _ in 0..BULK_COUNT {
                    let p = pool
                        .allocate::<i32>(LARGE_COUNT)
                        .expect("pool allocation failed");

                    unsafe {
                        for i in 0..LARGE_COUNT {
                            p.as_ptr().add(i).write(42);
                        }

                        black_box(p.as_ptr());

                        pool.deallocate(p.as_ptr());
                    }
                }
            });
        },
    );

    group.bench_function(
        BenchmarkId::new(
            "box_large_alloc_free",
            format!("{BULK_COUNT}x{LARGE_COUNT}"),
        ),
        |b| {
            b.iter(|| {
                for _ in 0..BULK_COUNT {
                    let value = Box::new([42i32; LARGE_COUNT]);
                    black_box(value);
                }
            });
        },
    );

    // ============================================================
    // 3. 大量 allocation 同时存活
    // ============================================================
    //
    // 注意：
    // pool 必须放在 b.iter() 外。
    //
    // 否则每一轮都会：
    //
    //     allocate 10 MB backing storage
    //     ↓
    //     做 benchmark
    //     ↓
    //     dealloc 10 MB
    //
    // 这样测出来的就不是 allocation 本身了。
    //
    group.bench_function(BenchmarkId::new("pool_keep_alive", BULK_COUNT), |b| {
        let mut pool = create_pool();

        b.iter(|| {
            let mut ptrs = Vec::with_capacity(BULK_COUNT);

            for _ in 0..BULK_COUNT {
                let p = pool.allocate::<i32>(1).expect("pool allocation failed");

                unsafe {
                    p.as_ptr().write(42);
                }

                ptrs.push(p);
            }

            for p in ptrs {
                unsafe {
                    pool.deallocate(p.as_ptr());
                }
            }
        });
    });

    group.bench_function(BenchmarkId::new("box_keep_alive", BULK_COUNT), |b| {
        b.iter(|| {
            let mut values = Vec::with_capacity(BULK_COUNT);

            for _ in 0..BULK_COUNT {
                values.push(Box::new(42i32));
            }

            black_box(values);
        });
    });

    group.finish();
}
