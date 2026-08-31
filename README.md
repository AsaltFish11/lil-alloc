# lil-alloc

一个使用 Rust 编写的简易内存池分配器，用于学习和实践内存管理底层原理。

## 功能特性

- 基于空闲链表的内存块管理
- 支持分配（`allocate`）与释放（`deallocate`）
- 释放时自动调用类型析构函数（`drop_in_place`）
- 释放后尝试与相邻空闲块合并，减少碎片
- 面向对象风格 API，封装内部状态

## 使用示例

```rust
use lil_alloc::MemoryPool;
use std::alloc::Layout;

let pool = MemoryPool::new(Layout::from_size_align(1024, 8).unwrap());

// 分配 10 个 i32 的内存
let ptr = pool.allocate::<i32>(10).unwrap();
unsafe {
    ptr.as_ptr().write(42);
    assert_eq!(*ptr.as_ptr(), 42);
    pool.deallocate(ptr); // 释放并析构
}
```

## 测试

运行所有测试（包括文档测试）：

```bash
cargo test
```

当前测试覆盖：
- 基本分配与释放
- 内存不足时返回 `None`
- 释放空指针的安全性
- 释放后内存可重用

注意: 部分测试代码由AI生成

## 性能基准

运行基准测试（需要 Rust nightly）：
注意: 性能测试代码由AI生成

```bash
cargo bench
```

## 后续计划

- 修复碎片化场景下的分配失败问题
- 支持动态扩容
- 添加对齐约束检查
- 支持多线程安全
- 针对大量小内存分配进行优化

## 许可
```text
MIT License

Copyright (c) 2026 AsaltFish11

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
