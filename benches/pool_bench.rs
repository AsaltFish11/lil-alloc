// benches/pool_bench.rs
extern crate std;
use criterion::*;
use lil_alloc::MemoryPool;
use std::alloc::Layout;

fn bench_single_allocation(c: &mut Criterion) {
    let pool = MemoryPool::new(Layout::from_size_align(1024 * 1024, 8).unwrap());

    // 1. 单次小批量：分配1个
    c.bench_function("pool single 1 alloc", |b| {
        b.iter(|| {
            let p = pool.allocate::<i32>(1).unwrap();
            unsafe {
                pool.deallocate(p.as_ptr());
            }
        })
    });

    c.bench_function("box single 1 alloc", |b| {
        b.iter(|| {
            let v = Box::new(42);
            drop(v);
        })
    });
}

fn bench_bulk_allocation(c: &mut Criterion) {
    let pool = MemoryPool::new(Layout::from_size_align(1024 * 1024, 8).unwrap());
    const BULK_SIZE: usize = 1000;

    // 2. 单次大批量：分配1000个
    c.bench_function(&format!("pool bulk {} allocs", BULK_SIZE), |b| {
        b.iter(|| {
            let p = pool.allocate::<i32>(BULK_SIZE).unwrap();
            unsafe {
                pool.deallocate(p.as_ptr());
            }
        })
    });

    c.bench_function(&format!("box bulk {} allocs", BULK_SIZE), |b| {
        b.iter(|| {
            let mut boxes = Vec::with_capacity(BULK_SIZE);
            for _ in 0..BULK_SIZE {
                boxes.push(Box::new(42));
            }
            drop(boxes); // 自动释放
            // Box: 你, 这不公平, 你咋不初始化
        })
    });
}

// 两个独立的 benchmark group
criterion_group!(single, bench_single_allocation);
criterion_group!(bulk, bench_bulk_allocation);
criterion_main!(single, bulk);
