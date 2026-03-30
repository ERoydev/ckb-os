#![no_std]

mod memory;
mod syscalls;

pub use memory::init_memory;
pub use syscalls::syscall_handle;
