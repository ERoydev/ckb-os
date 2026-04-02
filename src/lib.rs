#![no_std]

mod allocator;
mod errno;
mod handlers;
mod mman;
#[cfg(all(target_arch = "riscv64", target_os = "linux"))]
mod riscv64;
#[cfg(all(target_arch = "riscv64", target_os = "linux"))]
mod stack;
mod syscalls;
#[cfg(all(target_arch = "riscv64", target_os = "linux"))]
mod trap;

pub use allocator::init as init_heap;
pub use errno::*;
pub use syscalls::syscall_handle;
