#![no_std]

mod allocator;
#[cfg(all(target_arch = "riscv64", target_os = "linux"))]
pub mod bootstrap;
mod errno;
mod handlers;
mod mman;
#[cfg(all(target_arch = "riscv64", target_os = "linux"))]
mod riscv64;
mod syscalls;
mod stack;

pub use allocator::init as init_heap;
pub use errno::*;
pub use syscalls::syscall_handle;
