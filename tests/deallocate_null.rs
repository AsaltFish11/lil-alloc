use lil_alloc::MemoryPool;
use std::alloc::Layout;

#[test]
fn test_deallocate_null() {
    let layout = Layout::from_size_align(1024, 8).unwrap();
    let mut pool = MemoryPool::new(layout);

    let null_ptr: *mut i32 = std::ptr::null_mut();
    unsafe {
        // 文档保证传递空指针是安全的，不会 panic 或崩溃
        pool.deallocate(null_ptr);
    }
}
