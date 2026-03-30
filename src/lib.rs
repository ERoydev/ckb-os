#![no_std]

#[cfg(target_arch = "riscv64")]
pub mod bootstrap;
mod memory;
mod syscalls;

pub use memory::init_memory;
pub use syscalls::syscall_handle;
