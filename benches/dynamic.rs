use criterion::Criterion;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use crate::common::{create_pool, generate_sizes};

pub fn bench_dynamic_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("dynamic_allocation");

    // ============================================================
    // 所有 benchmark 共用完全相同的 sizes
    // ============================================================
    let sizes = generate_sizes(10_000);

    // ============================================================
    // 1. MemoryPool 动态大小
    // ============================================================
    group.bench_function("pool_dynamic_sizes", |b| {
        let mut pool = create_pool();

        b.iter(|| {
            let mut ptrs = Vec::with_capacity(sizes.len());

            for &size in &sizes {
                let p = pool.allocate::<i32>(size).expect("pool allocation failed");

                unsafe {
                    for i in 0..size {
                        p.as_ptr().add(i).write(42);
                    }
                }

                ptrs.push(p);
            }

            for p in ptrs.into_iter().rev() {
                unsafe {
                    pool.deallocate(p.as_ptr());
                }
            }
        });
    });

    // ============================================================
    // 2. Box 动态大小
    //
    // 使用 Box<[i32]>，长度真正等于 size。
    // ============================================================
    group.bench_function("box_dynamic_sizes", |b| {
        b.iter(|| {
            let mut values = Vec::with_capacity(sizes.len());

            for &size in &sizes {
                let value = vec![42i32; size].into_boxed_slice();
                values.push(value);
            }

            drop(values);
        });
    });

    // ============================================================
    // 3. 固定操作序列的 alternating alloc/free
    // ============================================================

    #[derive(Clone, Copy)]
    enum Operation {
        Alloc,
        Free,
    }

    // benchmark 外生成一次。
    // 不在 b.iter() 里调用 rand。
    let operations = {
        let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF);

        let mut ops = Vec::with_capacity(100);

        let mut alive = 0usize;

        for _ in 0..100 {
            let operation = if alive == 0 {
                Operation::Alloc
            } else if alive >= 50 {
                Operation::Free
            } else if rng.random_bool(0.5) {
                Operation::Alloc
            } else {
                Operation::Free
            };

            match operation {
                Operation::Alloc => alive += 1,
                Operation::Free => alive -= 1,
            }

            ops.push(operation);
        }

        ops
    };

    // ------------------------------------------------------------
    // Pool
    // ------------------------------------------------------------
    group.bench_function("pool_alternating_alloc_free", |b| {
        let mut pool = create_pool();

        b.iter(|| {
            let mut ptrs = Vec::with_capacity(100);

            for operation in &operations {
                match operation {
                    Operation::Alloc => {
                        let p = pool.allocate::<i32>(64).expect("pool allocation failed");

                        unsafe {
                            p.as_ptr().write(42);
                        }

                        ptrs.push(p);
                    }

                    Operation::Free => {
                        let p = ptrs.pop().expect("invalid operation sequence");

                        unsafe {
                            pool.deallocate(p.as_ptr());
                        }
                    }
                }
            }

            for p in ptrs {
                unsafe {
                    pool.deallocate(p.as_ptr());
                }
            }
        });
    });

    // ------------------------------------------------------------
    // Box
    // ------------------------------------------------------------
    group.bench_function("box_alternating_alloc_free", |b| {
        b.iter(|| {
            let mut values = Vec::with_capacity(100);

            for operation in &operations {
                match operation {
                    Operation::Alloc => {
                        values.push(vec![42i32; 64].into_boxed_slice());
                    }

                    Operation::Free => {
                        drop(values.pop());
                    }
                }
            }

            drop(values);
        });
    });

    group.finish();
}
