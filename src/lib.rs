#![no_std]

#[cfg(all(target_arch = "riscv64", target_os = "linux"))]
pub mod bootstrap;
mod memory;
mod syscalls;

pub use memory::init_memory;
pub use syscalls::syscall_handle;
