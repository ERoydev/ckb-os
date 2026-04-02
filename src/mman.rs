// ref -> https://github.com/torvalds/linux/blob/master/include/uapi/asm-generic/mman-common.h
pub const MAP_ANONYMOUS: usize = 0x20;
pub const MAP_PRIVATE: usize = 0x02;

pub const PROT_NONE: usize = 0x0;
pub const PROT_READ: usize = 0x1;
pub const PROT_WRITE: usize = 0x2;
// pub const PROT_EXEC: usize = 0x4;

pub const MAP_STACK: usize = 0x020000;
