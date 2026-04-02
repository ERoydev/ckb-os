use crate::errno::ENOSYS;
use crate::handlers;

// Syscall numbers (riscv64 Linux ABI)
const SYS_EXIT: usize = 93;
const SYS_EXIT_GROUP: usize = 94;
const SYS_TKILL: usize = 130;
const SYS_TGKILL: usize = 131;
const SYS_RT_SIGACTION: usize = 134;
const SYS_RT_SIGPROCMASK: usize = 135;
const SYS_WRITE: usize = 64;
const SYS_WRITEV: usize = 66;
const SYS_BRK: usize = 214;
const SYS_MUNMAP: usize = 215;
const SYS_MMAP: usize = 222;
const SYS_MPROTECT: usize = 226;

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
        SYS_BRK => handlers::sys_brk(a0),
        SYS_MMAP => handlers::sys_mmap(a0, a1, a2, a3, a4, a5),
        SYS_MUNMAP => handlers::sys_munmap(a0, a1),
        SYS_MPROTECT => handlers::sys_mprotect(a0, a1, a2),

        // I/O — write(fd, buf, count): pretend all bytes were written
        SYS_WRITE => handlers::sys_write(a0, a1, a2),
        SYS_WRITEV => handlers::sys_writev(a0, a1, a2),

        // Process
        SYS_EXIT => handlers::sys_exit(a0),
        SYS_EXIT_GROUP => handlers::sys_exit_group(a0),

        // Signals (stubs)
        SYS_RT_SIGACTION => handlers::sys_rt_sigaction(),
        SYS_RT_SIGPROCMASK => handlers::sys_rt_sigprocmask(),
        SYS_TKILL => handlers::sys_tkill(a1),
        SYS_TGKILL => handlers::sys_tgkill(a2),

        // Everything else
        _ => ENOSYS,
    }
}
