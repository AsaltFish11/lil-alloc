use lil_alloc::MemoryPool;
use std::alloc::Layout;

#[test]
fn test_allocate_fail() {
    // 池子只有 64 字节
    let layout = Layout::from_size_align(64, 8).unwrap();
    let mut pool = MemoryPool::new(layout);

    // 尝试分配 100 个 u8（需要 100 字节），应返回 None
    let ptr = pool.allocate::<u8>(100);
    assert!(ptr.is_none(), "分配超出内存池大小应失败");
}
