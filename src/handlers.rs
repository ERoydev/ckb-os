use crate::memory;

unsafe extern "C" {
    fn __platform_exit(code: u32) -> !;
    fn __platform_abort(sig: i32) -> !;
}

// Memory
pub fn sys_brk(a0: usize) -> isize {
    memory::sys_brk(a0)
}

pub fn sys_mmap(a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> isize {
    memory::sys_mmap(a0, a1, a2, a3, a4, a5)
}

pub fn sys_munmap(a0: usize, a1: usize) -> isize {
    memory::sys_munmap(a0, a1)
}

pub fn sys_mprotect(a0: usize, a1: usize, a2: usize) -> isize {
    memory::sys_mprotect(a0, a1, a2)
}

// Process exit
pub fn sys_exit(code: usize) -> ! {
    unsafe { __platform_exit(code as u32) }
}

pub fn sys_exit_group(code: usize) -> ! {
    unsafe { __platform_exit(code as u32) }
}

// Signals (stubs)
pub fn sys_rt_sigaction() -> isize {
    0
}

pub fn sys_rt_sigprocmask() -> isize {
    0
}

pub fn sys_tkill(sig: usize) -> ! {
    unsafe { __platform_abort(sig as i32) }
}

pub fn sys_tgkill(sig: usize) -> ! {
    unsafe { __platform_abort(sig as i32) }
}

// write(fd, buf, count) → count (pretend all bytes written)
pub fn sys_write(_fd: usize, _buf: usize, count: usize) -> isize {
    count as isize
}

// writev(fd, iov, iovcnt) → sum of all iov_len
pub fn sys_writev(_fd: usize, iov: usize, iovcnt: usize) -> isize {
    // iovec is { *buf, len } — 2 usizes each
    let mut total = 0usize;
    for i in 0..iovcnt {
        let len_ptr = iov + i * 16 + 8; // offset to iov_len field                                                                                                                        
        let len = unsafe { core::ptr::read(len_ptr as *const usize) };
        total += len;
    }
    total as isize
}

pub fn sys_ioctl() -> isize {
    -25 // ENOTTY
}

// Process init
pub fn sys_set_tid_address() -> isize {
    0
}

// Time — zero timespec for deterministic ZK execution
pub fn sys_clock_gettime(timespec_ptr: usize) -> isize {
    if timespec_ptr != 0 {
        let ts = timespec_ptr as *mut [u64; 2];
        unsafe { core::ptr::write(ts, [0u64; 2]) };
    }
    0
}

// Memory hints
pub fn sys_madvise() -> isize {
    0
}
