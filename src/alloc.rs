extern crate alloc;

use super::block::Block;
use super::memorypool::MemoryPool;
use alloc::alloc::{Layout, alloc};
use core::ptr::{NonNull, null_mut};

impl MemoryPool {
    pub fn new(layout: Layout) -> Self {
        let mut pool = Self {
            begin_block: null_mut(),
        };
        unsafe {
            let block_ptr = alloc(layout) as *mut Block;
            // 申请初始内存块都不行, 玩不了了
            if !block_ptr.is_null() {
                (*block_ptr).free = true;
                (*block_ptr).prev = null_mut();
                (*block_ptr).next = null_mut();
                (*block_ptr).size = layout.size();
                pool.begin_block = block_ptr
            } else {
                panic!("alloc pool fail");
            }
        }
        pool
    }

    /// 为类型 `T` 分配一块连续内存，可容纳 `count` 个元素。
    ///
    /// # 参数
    /// - `count`: 需要分配的 `T` 类型元素个数。
    ///
    /// # 返回值
    /// 返回 `Option<NonNull<T>>`：
    /// - 若分配成功，返回指向该内存块起始地址的 `NonNull<T>` 指针；
    /// - 若分配失败（如内存不足或布局无效），返回 `None`。
    ///
    /// # 安全性
    /// - 调用者需确保返回的指针在使用期间不会发生别名冲突或越界访问；
    /// - 该分配器不自动管理内存生命周期，调用者需在适当时机手动释放内存。
    ///
    /// # Panics
    /// 当 `count * size_of::<T>()` 溢出 `usize` 时，该函数会触发 panic。
    ///
    /// # 注意
    /// - 分配出的内存未初始化，调用者需自行初始化后再使用；
    /// - 当前实现中，若空闲链表无合适块，会返回 `None`（扩容功能尚未实现）。
    ///
    /// # 示例
    /// ```
    /// use lil_alloc::MemoryPool;
    /// use std::alloc::Layout;
    /// 
    /// let pool = MemoryPool::new(Layout::from_size_align(1024, 8).unwrap());
    /// let ptr = pool.allocate::<u32>(4);
    /// assert!(ptr.is_some());
    /// ```
    pub fn allocate<T: Sized>(&self, count: usize) -> Option<NonNull<T>> {
        let _ = size_of::<T>().checked_mul(count).expect(
            "The request size should be small enough that size * T::SIZE does not overflow a usize",
        );
        let layout = Layout::array::<T>(count).expect("size * T::SIZE should not overflow usize");
        unsafe { NonNull::new(self.alloc_u8(layout) as *mut T) }
    }

    /// 返回一个大小至少为 size + size_of::<Block>() 的 Block 指针
    /// 失败返回空指针
    unsafe fn get_free_block(&self, size: usize) -> *mut Block {
        unsafe {
            let mut ptr = self.begin_block;
            while !ptr.is_null() {
                if !(*ptr).free {
                    ptr = (*ptr).next;
                    continue;
                }
                // 直接分配
                if size <= (*ptr).size && (*ptr).size <= size + size_of::<Block>() {
                    return ptr;
                } else if (*ptr).size > size + size_of::<Block>() {
                    // 太大直接全给出去浪费
                    if (*ptr).size > size + 2 * size_of::<Block>() {
                        let a_ptr = ptr;
                        let b_ptr = (ptr.add(1) as *mut u8).add(size) as *mut Block;
                        let next_ptr = (*a_ptr).next;
                        // 补全 prev 和 next
                        (*a_ptr).next = b_ptr;
                        (*b_ptr).prev = a_ptr;
                        (*b_ptr).next = next_ptr;
                        // 修改大小
                        let old_size = (*a_ptr).size;
                        (*a_ptr).size = size;
                        (*b_ptr).size = old_size - size - size_of::<Block>();
                        // 补全 free
                        (*b_ptr).free = true;
                        return a_ptr;
                    }
                    // 直接全给出去
                    else {
                        return ptr;
                    }
                } else {
                    ptr = (*ptr).next;
                }
            }
            null_mut()
        }
    }

    unsafe fn alloc_u8(&self, layout: Layout) -> *mut u8 {
        if self.begin_block.is_null() {
            return null_mut();
        }
        // 在链表中找到一块符合大小的空闲块,拆出来一个指定大小并将多出来的插回去
        unsafe {
            let ptr = self.get_free_block(layout.size());
            // 如果空闲链表中没有符合的, 则需要扩容
            // 但是先不实现, 返回一个空指针
            if ptr.is_null() {
                return null_mut();
            }
            (*ptr).free = false;
            return ptr.add(1) as *mut u8;
        }
    }
}
