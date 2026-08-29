// #![no_std]
extern crate alloc;

use alloc::alloc::{Layout, alloc};
use core::ptr::null_mut;

const POOL_INIT_SIZE: usize = 1 * 1024 * 1024; // 1MiB大小
const POOL_ALIGN: usize = size_of::<Block>();

struct Block {
    // 不包含Block的所指向的块的大小
    size: usize,
    // true 表示可以用, false 表示被占用不能用
    free: bool,
    next: *mut Block,
    prev: *mut Block,
}

static mut FREELIST_HEAD: *mut Block = null_mut(); // 表示空闲链表表头的 Block

/// 空闲内存块列表
unsafe fn init_free_pool() {
    let layout = Layout::from_size_align(POOL_INIT_SIZE + size_of::<Block>(), POOL_ALIGN);
    if let Ok(layout) = layout {
        unsafe {
            let block_ptr = alloc(layout) as *mut Block;
            (*block_ptr).free = true;
            (*block_ptr).prev = null_mut();
            (*block_ptr).next = null_mut();
            (*block_ptr).size = POOL_INIT_SIZE;
            FREELIST_HEAD = block_ptr
        }
    }
}

/// 将两个Block进行合并并将 free 状态设置为 a_ptr 的 free 状态, 并返回剩余的那个指针
unsafe fn marge_two_block(a_ptr: *mut Block, b_ptr: *mut Block) -> *mut Block {
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
unsafe fn marge(ptr: *mut Block) {
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
            ptr = marge_two_block(p_ptr, ptr);
        }
        if !n_ptr.is_null() && (*n_ptr).free {
            marge_two_block(ptr, n_ptr);
        }
    }
}

/// 返回一个大小至少为 size + size_of::<Block>() 的 Block 指针
/// 失败返回空指针
unsafe fn get_free_block(size: usize) -> *mut Block {
    unsafe {
        let mut ptr = FREELIST_HEAD;
        while !ptr.is_null() {
            if !(*ptr).free {
                ptr = (*ptr).next;
                continue;
            }
            if (*ptr).size >= size + size_of::<Block>() {
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

pub unsafe fn alloc_u8(size: usize) -> *mut u8 {
    if !(size % 8 == 0) {
        return null_mut();
    }
    unsafe {
        if FREELIST_HEAD.is_null() {
            init_free_pool();
            // 内存申请失败
            if FREELIST_HEAD.is_null() {
                return null_mut();
            }
        }
    }
    // 在链表中找到一块符合大小的空闲块,拆出来一个指定大小并将多出来的插回去
    unsafe {
        let ptr = get_free_block(size);
        // 如果空闲链表中没有符合的, 则需要扩容
        // 但是先不实现, 返回一个空指针
        if ptr.is_null() {
            return null_mut();
        }
        (*ptr).free = false;
        return ptr.add(1) as *mut u8;
    }
}

/// 释放内存并合并周围可能存在的空闲块
pub unsafe fn dealloc_u8(ptr: *mut u8) {
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
        marge(block_ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试基本分配和释放
    #[test]
    fn test_basic_alloc_free() {
        let size = 64;
        let ptr = unsafe { alloc_u8(size) };
        assert!(!ptr.is_null(), "分配失败");

        unsafe {
            // 写入数据
            for i in 0..size {
                *ptr.add(i) = (i % 256) as u8;
            }
            // 验证数据
            for i in 0..size {
                assert_eq!(*ptr.add(i), (i % 256) as u8, "数据验证失败 at index {}", i);
            }
            dealloc_u8(ptr);
        }
    }

    /// 测试多次分配和释放（压力测试）
    #[test]
    fn test_multiple_alloc_free() {
        let sizes = [16, 32, 64, 128, 256, 512, 1024, 2048];

        for &size in &sizes {
            let ptr = unsafe { alloc_u8(size) };
            assert!(!ptr.is_null(), "分配 size={} 失败", size);

            unsafe {
                // 写入模式：用索引的低8位
                for i in 0..size {
                    *ptr.add(i) = (i & 0xFF) as u8;
                }
                // 验证
                for i in 0..size {
                    assert_eq!(
                        *ptr.add(i),
                        (i & 0xFF) as u8,
                        "size={} 验证失败 at {}",
                        size,
                        i
                    );
                }
                dealloc_u8(ptr);
            }
        }
    }

    /// 测试分配后不释放（耗尽池子）
    #[test]
    fn test_alloc_until_exhaustion() {
        let mut allocated = Vec::new();
        let mut total = 0;

        // 分配 64 字节块，直到分配器返回空指针
        while let Some(ptr) = unsafe {
            let p = alloc_u8(64);
            if p.is_null() { None } else { Some(p) }
        } {
            unsafe {
                // 写入一些数据，确保内存可访问
                *ptr.add(0) = 0xAA;
                *ptr.add(63) = 0x55;
            }
            allocated.push(ptr);
            total += 1;
        }

        // 至少应该能分配一些块
        assert!(
            total > 10,
            "至少应该能分配10个以上的块，实际只分配了 {}",
            total
        );

        // 释放所有分配的内存
        for ptr in allocated {
            unsafe {
                dealloc_u8(ptr);
            }
        }
    }

    /// 测试释放后内存可复用（验证合并逻辑）
    #[test]
    fn test_reuse_after_free() {
        // 分配两个块
        let ptr1 = unsafe { alloc_u8(1024) };
        let ptr2 = unsafe { alloc_u8(1024) };
        assert!(!ptr1.is_null() && !ptr2.is_null(), "初始分配失败");

        // 释放第一个块
        unsafe {
            dealloc_u8(ptr1);
        }

        // 再分配一个同样大小的块，应该能复用ptr1的内存
        let ptr3 = unsafe { alloc_u8(1024) };

        assert!(!ptr3.is_null(), "释放后复用失败");
        assert_eq!(ptr1, ptr3, "释放后没有复用同一块内存");

        unsafe {
            dealloc_u8(ptr2);
            dealloc_u8(ptr3);
        }
    }

    /// 测试随机大小的分配释放（模拟真实场景）
    #[test]
    fn test_random_sizes() {
        let sizes: [usize; 16] = [
            12, 24, 36, 48, 60, 72, 84, 96, 108, 120, 132, 144, 156, 168, 180, 192,
        ];
        let mut ptrs = [core::ptr::null_mut(); 16];

        // 分配所有块
        for (i, &size) in sizes.iter().enumerate() {
            ptrs[i] = unsafe { alloc_u8(size) };
            assert!(!ptrs[i].is_null(), "分配 size={} 失败", size);
            unsafe {
                for j in 0..size {
                    *ptrs[i].add(j) = (j & 0xFF) as u8;
                }
            }
        }

        // 释放一半的块
        for i in (0..16).step_by(2) {
            unsafe {
                dealloc_u8(ptrs[i]);
            }
            ptrs[i] = core::ptr::null_mut();
        }

        // 重新分配那些块，看能否复用
        for (i, &size) in sizes.iter().enumerate().step_by(2) {
            ptrs[i] = unsafe { alloc_u8(size) };
            assert!(!ptrs[i].is_null(), "重新分配 size={} 失败", size);
            unsafe {
                for j in 0..size {
                    *ptrs[i].add(j) = (j & 0xFF) as u8;
                }
            }
        }

        // 释放所有块
        for ptr in ptrs.iter().filter(|&&p| !p.is_null()) {
            unsafe {
                dealloc_u8(*ptr);
            }
        }
    }
}
