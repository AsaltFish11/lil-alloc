use criterion::Criterion;

use crate::common::create_pool;

pub fn bench_fragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("fragmentation");

    group.bench_function("pool_fragmentation", |b| {
        let mut pool = create_pool();

        b.iter(|| {
            let mut ptrs = Vec::with_capacity(128);

            // ====================================================
            // 第一阶段：分配许多大小不同的块
            // ====================================================
            for i in 0..128usize {
                let count = match i % 4 {
                    0 => 8,
                    1 => 16,
                    2 => 32,
                    _ => 64,
                };

                let p = pool.allocate::<i32>(count).expect("pool allocation failed");

                unsafe {
                    p.as_ptr().write(42);
                }

                ptrs.push(p);
            }

            // ====================================================
            // 第二阶段：释放一部分
            // ====================================================
            let mut free = Vec::new();

            for i in (0..ptrs.len()).step_by(2) {
                free.push(ptrs[i]);
            }

            for p in free {
                unsafe {
                    pool.deallocate(p.as_ptr());
                }
            }

            // ====================================================
            // 第三阶段：利用碎片重新分配
            // ====================================================
            for _ in 0..32 {
                let p = pool
                    .allocate::<i32>(16)
                    .expect("fragmented allocation failed");

                unsafe {
                    p.as_ptr().write(42);
                    pool.deallocate(p.as_ptr());
                }
            }

            // ====================================================
            // 第四阶段：清理剩余 allocation
            // ====================================================
            for (i, p) in ptrs.into_iter().enumerate() {
                if i % 2 == 1 {
                    unsafe {
                        pool.deallocate(p.as_ptr());
                    }
                }
            }
        });
    });

    group.finish();
}
