extern crate alloc;

use super::block::Block;
use super::memorypool::MemoryPool;
use alloc::alloc::Layout;
use core::ptr::{NonNull, null_mut};

impl MemoryPool {
    /// 为类型 `T` 分配一块连续内存，可容纳 `count` 个元素 ( `count` 不为0)。
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
    /// - 当`count`为`0`时, 不执行分配, 返回None
    /// - 请求的字节对齐不能超过池子能保证的对齐（创建池时的 `Layout.align()`，上限为 `size_of::<Block>()`），
    ///   超出时返回 `None`。例如池用 8 对齐创建时，`u128`(16 对齐) 会返回 `None`；用 16 对齐创建即可支持。
    ///
    /// # 示例
    /// ```
    /// use lil_alloc::MemoryPool;
    /// use std::alloc::Layout;
    ///
    /// let mut pool = MemoryPool::new(Layout::from_size_align(1024, 8).unwrap());
    /// let ptr = pool.allocate::<u32>(4);
    /// assert!(ptr.is_some());
    /// ```
    pub fn allocate<T: Sized>(&mut self, count: usize) -> Option<NonNull<T>> {
        if count == 0 {
            return None;
        }
        let layout = Layout::array::<T>(count).expect("size * T::SIZE should not overflow usize");
        // 数据区 = 块头 + Block 大小, 所以能保证的请求对齐 = min(池对齐, Block 大小)
        if layout.align() > self.align().min(size_of::<Block>()) {
            return None;
        }
        unsafe { NonNull::new(self.alloc_u8(layout) as *mut T) }
    }

    /// 返回一个大小至少为 size + size_of::<Block>() 的 Block 指针
    /// 失败返回空指针
    unsafe fn get_free_block(&mut self, size: usize) -> *mut Block {
        unsafe {
            let mut ptr = self.begin_block();
            while !ptr.is_null() {
                if !(*ptr).free {
                    ptr = (*ptr).next;
                    continue;
                }
                // 直接分配
                if size <= (*ptr).size && (*ptr).size <= size + 2 * size_of::<Block>() {
                    (*ptr).free = false;
                    return ptr;
                }
                // 太大直接全给出去浪费
                else if (*ptr).size > size + 2 * size_of::<Block>() {
                    let a_ptr = ptr;
                    // 新块头按池的对齐取整 (不是固定 8!), 这样块头地址始终是池对齐的倍数,
                    // 数据区 = 块头 + Block 也就满足请求的对齐要求
                    let b_ptr_addr = (size_of::<Block>() + size).next_multiple_of(self.align());
                    let padding = b_ptr_addr - size_of::<Block>() - size;
                    let b_ptr = (ptr as *mut u8).add(b_ptr_addr) as *mut Block;
                    let next_ptr = (*a_ptr).next;
                    // 补全 prev 和 next
                    (*a_ptr).next = b_ptr;
                    (*b_ptr).prev = a_ptr;
                    (*b_ptr).next = next_ptr;
                    if !next_ptr.is_null() {
                        (*next_ptr).prev = b_ptr;
                    }
                    // 修改大小: padding 算进 a, b 的大小减去 padding, 保证数据区不越过池尾
                    let old_size = (*a_ptr).size;
                    (*a_ptr).size = size + padding;
                    (*b_ptr).size = old_size - size - padding - size_of::<Block>();
                    // 补全 free
                    (*b_ptr).free = true;
                    return a_ptr;
                } else {
                    ptr = (*ptr).next;
                }
            }
            null_mut()
        }
    }

    unsafe fn alloc_u8(&mut self, layout: Layout) -> *mut u8 {
        if self.begin_block().is_null() {
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
            ptr.add(1) as *mut u8
        }
    }
}
