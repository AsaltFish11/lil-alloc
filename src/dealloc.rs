use super::block::Block;
use super::memorypool::MemoryPool;

impl MemoryPool {
    /// 释放由 `allocate` 分配的内存
    ///
    /// # 参数
    /// - `ptr`: 指向需要释放的内存块的指针，可以是 `*mut T` 或任何实现了 `Into<*mut T>` 的类型
    ///
    /// # 行为
    /// **释放内存**：将原始内存块标记为空闲，并尝试与相邻的空闲块合并，以减少内存碎片。
    ///
    /// # Safety
    /// - **调用者必须保证**：
    ///   - `ptr` 指向的内存必须是由同一个 `MemoryPool` 实例的 `allocate` 方法分配出来的；
    ///   - `ptr` 指向的内存必须仍然有效，且未被释放过（即不能重复释放，double-free）；
    ///   - `ptr` 指向的对象必须已经初始化。
    /// - 传递空指针（`null_mut()`）是安全的
    ///
    /// # 注意
    /// - 该函数要求 `T: Sized`
    /// - 释放后，`ptr` 变为悬垂指针，调用者不应再使用它；
    /// - 合并操作仅当相邻块为空闲时发生，有助于后续分配的连续性。
    ///
    /// # 示例
    /// ```
    /// use lil_alloc::MemoryPool;
    /// use std::alloc::Layout;
    ///
    /// let mut pool = MemoryPool::new(Layout::from_size_align(1024, 8).unwrap());
    /// let ptr = pool.allocate::<i32>(1).unwrap();
    /// unsafe {
    ///     ptr.as_ptr().write(145);
    ///     pool.deallocate(ptr.as_ptr());
    /// }
    /// ```
    pub unsafe fn deallocate<T, P>(&mut self, ptr: P)
    where
        T: Sized,
        P: Into<*mut T>,
    {
        let ptr = ptr.into();
        if ptr.is_null() {
            return;
        }
        unsafe {
            self.dealloc_u8(ptr as *mut u8);
        }
    }

    /// 将两个Block进行合并, 并返回剩余的那个指针
    unsafe fn marge_two_block(&self, a_ptr: *mut Block, b_ptr: *mut Block) -> *mut Block {
        unsafe {
            let next_ptr = (*b_ptr).next;
            let size = (*a_ptr).size + (*b_ptr).size + size_of::<Block>();
            (*a_ptr).next = next_ptr;
            (*a_ptr).size = size;
            if !next_ptr.is_null() {
                (*next_ptr).prev = a_ptr;
            }
            a_ptr
        }
    }

    /// 将可能的左右两个空闲块进行合并
    unsafe fn marge(&self, ptr: *mut Block) {
        if ptr.is_null() {
            return;
        }
        unsafe {
            // 如果这儿块正在被占用, 不做任何事情
            if !(*ptr).free {
                return;
            }
            let p_ptr = (*ptr).prev; // 左侧block指针
            let n_ptr = (*ptr).next; // 右侧block指针
            let mut ptr = ptr;
            if !p_ptr.is_null() && (*p_ptr).free {
                ptr = self.marge_two_block(p_ptr, ptr);
            }
            if !n_ptr.is_null() && (*n_ptr).free {
                self.marge_two_block(ptr, n_ptr);
            }
        }
    }

    /// 释放内存并合并周围可能存在的空闲块
    unsafe fn dealloc_u8(&mut self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }
        let block_ptr = unsafe { (ptr as *mut Block).sub(1) };
        if block_ptr.is_null() {
            return;
        }
        unsafe {
            // 进行标记
            (*block_ptr).free = true;
            self.marge(block_ptr);
        }
    }
}
