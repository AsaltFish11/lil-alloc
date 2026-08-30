use lil_alloc::MemoryPool;
use std::alloc::Layout;

#[test]
fn test_basic_alloc_free() {
    // 创建一个 1KB 的内存池，对齐 8 字节
    let layout = Layout::from_size_align(1024, 8).unwrap();
    let pool = MemoryPool::new(layout);

    // 分配 10 个 i32（共 40 字节），应成功
    let ptr = pool.allocate::<i32>(10).unwrap();

    unsafe {
        // 写入数据并验证
        ptr.as_ptr().write(42);
        assert_eq!(*ptr.as_ptr(), 42);

        // 释放内存
        pool.deallocate(ptr.as_ptr());
    }
}
