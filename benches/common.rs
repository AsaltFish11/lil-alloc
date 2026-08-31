use lil_alloc::MemoryPool;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::alloc::Layout;

pub const POOL_SIZE: usize = 10 * 1024 * 1024;

pub const BULK_COUNT: usize = 10_000;

pub const LARGE_COUNT: usize = 4096;

pub const SMALL_COUNT: usize = 1;

pub const BLOCK: usize = 32;

/// 创建一个新的 MemoryPool。
pub fn create_pool() -> MemoryPool {
    let layout = Layout::from_size_align(POOL_SIZE, 8).expect("invalid memory pool layout");

    MemoryPool::new(layout)
}

/// 生成固定的动态 allocation size。
///
/// 这里故意使用固定 seed。
/// 这样 pool benchmark 和 Box benchmark 使用完全相同的输入。
pub fn generate_sizes(count: usize) -> Vec<usize> {
    let mut rng = StdRng::seed_from_u64(0x1234_5678);

    let mut sizes = Vec::with_capacity(count);
    let mut total_bytes = 0usize;

    for _ in 0..count {
        let size = rng.random_range(1..=LARGE_COUNT);
        let bytes = size * std::mem::size_of::<i32>();

        if total_bytes + bytes > POOL_SIZE {
            break;
        }

        sizes.push(size);
        total_bytes += bytes + BLOCK;
    }
    sizes
}
