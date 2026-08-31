extern crate alloc;

use crate::block::Block;
use alloc::alloc::{Layout, alloc, dealloc};
use core::ptr::null_mut;

/// 一个基于空闲链表（Free List）的内存池，用于高效管理固定内存区域中的动态分配。
///
/// `MemoryPool` 通过在预分配的内存块上维护一个双向空闲链表，实现了内存的分配与释放。
/// 它支持合并相邻空闲块以减少碎片，但不支持自动扩容（超出初始容量时分配将失败）。
///
/// # 内存布局
/// 每个内存块（`Block`）包含：
/// - 元数据（前驱/后继指针、大小、空闲标志）
/// - 紧随其后的可用数据区域
///
/// 分配时，从链表中找到一个足够大的空闲块，并根据需要将其拆分为已分配块和剩余空闲块。
/// 释放时，将块标记为空闲并尝试与左右相邻空闲块合并。
///
/// # 安全性
/// - 该内存池**不是线程安全的**, 目前不支持多线程安全
/// - `allocate` 返回的指针在未调用 `deallocate` 前有效，调用者需遵守所有权的管理规则。
/// - 该池不实现自动扩容，若空闲块不足，`allocate` 会返回 `None`。
///
/// # 示例
/// ```
/// use core::alloc::Layout;
/// use lil_alloc::MemoryPool;
///
/// // 创建一个容量为 4096 字节、对齐为 8 的内存池
/// let layout = Layout::from_size_align(4096, 8).unwrap();
/// let mut pool = MemoryPool::new(layout);
///
/// // 分配 4 个 u32 元素（16 字节）
/// let ptr = pool.allocate::<u32>(4).unwrap();
/// unsafe {
///     ptr.as_ptr().write(42);
///     assert_eq!(*ptr.as_ptr(), 42);
/// }
///
/// // 释放内存（自动运行析构函数）
/// unsafe {
///     pool.deallocate(ptr.as_ptr());
/// }
/// ```
///
/// # 注意
/// - 当前版本不支持扩容，分配请求超过剩余可用空间时将返回 `None`；
/// - 内存对齐要求由传入的 `Layout` 决定，需确保分配大小与块元数据不冲突；
/// - 释放时需要确保传入的指针来自同一内存池，否则行为未定义。
#[derive(Debug)]
pub struct MemoryPool {
    /// 指向内存池中第一个块（`Block`）的指针。
    /// 若为空指针（`null_mut()`），则表示内存池未初始化或已耗尽。
    begin_block: *mut Block,

    layout: Layout,
}

impl MemoryPool {
    pub fn new(layout: Layout) -> Self {
        let layout = Layout::from_size_align(
            layout.size() + size_of::<Block>(),
            layout.align().max(align_of::<Block>()),
        )
        .expect("The layout size is too large or the alignment is not a power of two; you should reduce the layout size or adjust the alignment.");
        let mut pool = Self {
            begin_block: null_mut(),
            layout,
        };
        unsafe {
            let block_ptr = alloc(layout) as *mut Block;
            // 申请初始内存块都不行, 玩不了了
            if !block_ptr.is_null() {
                (*block_ptr).free = true;
                (*block_ptr).prev = null_mut();
                (*block_ptr).next = null_mut();
                (*block_ptr).size = layout.size() - size_of::<Block>();
                pool.begin_block = block_ptr;
            } else {
                panic!("alloc pool fail");
            }
        }
        pool
    }

    pub fn begin_block(&self) -> *mut Block {
        self.begin_block
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.begin_block as *mut u8, self.layout);
        }
    }
}
