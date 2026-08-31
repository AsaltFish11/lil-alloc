//! 对齐与池尾边界的回归测试
use lil_alloc::MemoryPool;
use std::alloc::Layout;

/// 8 对齐池: u8 之后分配 u64 仍必须 8 对齐
#[test]
fn u8_then_u64_ok() {
    let mut pool = MemoryPool::new(Layout::from_size_align(1024, 8).unwrap());
    let _p0 = pool.allocate::<u8>(1).unwrap();
    let p = pool.allocate::<u64>(1).unwrap();
    assert_eq!(p.as_ptr() as usize % 8, 0, "u64 必须 8 对齐");
    unsafe { pool.deallocate(p.as_ptr()) };
}

/// 16 对齐池: 支持 u128, 且交错分配后依然 16 对齐
#[test]
fn pool16_supports_u128() {
    let mut pool = MemoryPool::new(Layout::from_size_align(1024, 16).unwrap());
    // 制造非 16 倍数的偏移
    let _p0 = pool.allocate::<u8>(1).unwrap();
    let p = pool.allocate::<u128>(1).unwrap();
    assert_eq!(p.as_ptr() as usize % 16, 0, "u128 必须 16 对齐");
    let _p1 = pool.allocate::<u8>(3).unwrap();
    let p2 = pool.allocate::<u128>(2).unwrap();
    assert_eq!(p2.as_ptr() as usize % 16, 0, "交错分配后仍 16 对齐");
    unsafe {
        pool.deallocate(p.as_ptr());
        pool.deallocate(p2.as_ptr());
    }
}

/// 8 对齐池: 16 对齐请求应被拒绝
#[test]
fn pool8_rejects_u128() {
    let mut pool = MemoryPool::new(Layout::from_size_align(1024, 8).unwrap());
    assert!(
        pool.allocate::<u128>(1).is_none(),
        "8 对齐池应拒绝 16 对齐请求"
    );
}

/// 整池分配 + 写满, 数据区不得越过池尾
#[test]
fn full_pool_write_stays_in_bounds() {
    let user = 1024usize;
    let mut pool = MemoryPool::new(Layout::from_size_align(user, 8).unwrap());
    let base = pool.begin_block() as usize;
    let pool_end = base + user + 32; // 后备内存 = user + Block 头

    let p = pool.allocate::<u8>(user).expect("整池分配应成功");
    let start = p.as_ptr() as usize;
    assert_eq!(start, base + 32, "数据区应在池基址 + Block 处");
    assert!(
        start + user <= pool_end,
        "数据区越界: {} > {}",
        start + user,
        pool_end
    );

    unsafe {
        p.as_ptr().write_bytes(0xAB, user); // 写满整池, 不得越界
        pool.deallocate(p.as_ptr());
    }
}

/// 奇数大小分配 -> 释放 -> 合并 -> 再分配大块
#[test]
fn odd_sizes_reuse_after_free() {
    let mut pool = MemoryPool::new(Layout::from_size_align(1024, 8).unwrap());
    let p1 = pool.allocate::<u8>(3).unwrap();
    let p2 = pool.allocate::<u8>(5).unwrap();
    unsafe {
        p1.as_ptr().write_bytes(1, 3);
        p2.as_ptr().write_bytes(2, 5);
        pool.deallocate(p1.as_ptr());
        pool.deallocate(p2.as_ptr());
    }
    // 释放并合并后应能再次分配大块
    let p3 = pool.allocate::<u8>(500).expect("合并后应能分配 500B");
    unsafe {
        p3.as_ptr().write_bytes(3, 500);
        pool.deallocate(p3.as_ptr());
    }
}
