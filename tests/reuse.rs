use lil_alloc::MemoryPool;
use std::alloc::Layout;

#[test]
fn test_memory_reuse() {
    let layout = Layout::from_size_align(1024, 8).unwrap();
    let mut pool = MemoryPool::new(layout);

    // 第一次分配
    let ptr1 = pool.allocate::<i32>(10).unwrap();
    let addr1 = ptr1.as_ptr() as usize;

    unsafe {
        pool.deallocate(ptr1.as_ptr()); // 释放，内存应该回到空闲链表
    }

    // 第二次分配（相同大小）
    let ptr2 = pool.allocate::<i32>(10).unwrap();
    let addr2 = ptr2.as_ptr() as usize;

    // 根据内存池的合并与重用策略，大概率会返回同一块地址
    // 若不相等，说明分配器没有重用刚释放的内存，这本身不一定是 bug，
    // 但作为基础测试，此处断言可以提示实现是否达到预期。
    assert_eq!(addr1, addr2, "释放后的内存应被重用");

    unsafe {
        pool.deallocate(ptr2.as_ptr());
    }
}
