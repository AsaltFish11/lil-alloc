pub struct Block {
    // 不包含Block的所指向的块的大小
    pub size: usize,
    // true 表示可以用, false 表示被占用不能用
    pub free: bool,
    pub next: *mut Block,
    pub prev: *mut Block,
}
