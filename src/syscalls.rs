use crate::memory;

const ENOSYS: isize = -38;

// Syscall numbers (riscv64 Linux ABI)
const SYS_EXIT: usize = 93;
const SYS_EXIT_GROUP: usize = 94;
const SYS_TKILL: usize = 130;
const SYS_TGKILL: usize = 131;
const SYS_RT_SIGACTION: usize = 134;
const SYS_RT_SIGPROCMASK: usize = 135;
const SYS_BRK: usize = 214;
const SYS_MUNMAP: usize = 215;
const SYS_MMAP: usize = 222;
const SYS_MPROTECT: usize = 226;

unsafe extern "C" {
    fn __platform_exit(code: u32) -> !;
    fn __platform_abort(sig: i32) -> !;
}

/// Dispatch a Linux syscall. Called from the ecall trap handler.
pub fn syscall_handle(
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    nr: usize,
) -> isize {
    match nr {
        // Memory
        SYS_BRK => memory::sys_brk(a0),
        SYS_MMAP => memory::sys_mmap(a0, a1, a2, a3, a4, a5),
        SYS_MUNMAP => memory::sys_munmap(a0, a1),
        SYS_MPROTECT => memory::sys_mprotect(a0, a1, a2),

        // Process exit
        SYS_EXIT => unsafe { __platform_exit(a0 as u32) },
        SYS_EXIT_GROUP => unsafe { __platform_exit(a0 as u32) },

        // Signals (stubs)
        SYS_RT_SIGACTION => 0,
        SYS_RT_SIGPROCMASK => 0,
        SYS_TKILL => unsafe { __platform_abort(a1 as i32) },
        SYS_TGKILL => unsafe { __platform_abort(a2 as i32) },

        // Everything else
        _ => ENOSYS,
    }
}
