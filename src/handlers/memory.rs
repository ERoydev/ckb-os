use crate::allocator;
use crate::errno::{EINVAL, ENOMEM};
use crate::mman::*;
use core::alloc::Layout;

const PAGE_SIZE: usize = 4096;

/// fails so musl falls back to mmap
pub fn sys_brk(_addr: usize) -> isize {
    ENOMEM
}

/// Decides what chunk of memory we need to allocate, asks the allocator and returns the pointer.
pub fn sys_mmap(
    addr: usize,
    len: usize,
    prot: usize,
    flags: usize,
    fd: usize,
    offset: usize,
) -> isize {
    if len == 0 {
        return EINVAL;
    }

    // TODO: Maybe some checks are not needed, but we can keep it strict for now
    let allowed_prot = PROT_NONE | PROT_READ | PROT_WRITE;
    if (prot & !allowed_prot) != 0 {
        return EINVAL;
    }

    let allowed_flags = MAP_PRIVATE | MAP_ANONYMOUS | MAP_STACK;
    if (flags & !allowed_flags) != 0 {
        return EINVAL;
    }

    if (flags & MAP_PRIVATE) == 0 || (flags & MAP_ANONYMOUS) == 0 {
        return EINVAL;
    }

    if addr != 0 || offset != 0 {
        return EINVAL;
    }

    if fd != usize::MAX && fd != 0 {
        return EINVAL;
    }

    let pages = len.div_ceil(PAGE_SIZE);
    let size = match pages.checked_mul(PAGE_SIZE) {
        Some(s) => s,
        None => return EINVAL,
    };

    // Describes the allocation request, (size, alignment), the returned pointer must be a multiple of the alignment.
    let layout = match Layout::from_size_align(size, PAGE_SIZE) {
        Ok(l) => l,
        Err(_) => return EINVAL,
    };

    let ptr = allocator::alloc(layout);
    if ptr.is_null() {
        return ENOMEM;
    }
    unsafe {
        core::ptr::write_bytes(ptr, 0, size);
    }
    ptr as isize
}

/// Reverse of mmap, releases the previously mapped memory region back
pub fn sys_munmap(addr: usize, len: usize) -> isize {
    if addr == 0 || len == 0 {
        return EINVAL;
    }
    if !addr.is_multiple_of(PAGE_SIZE) {
        return EINVAL;
    }

    let pages = len.div_ceil(PAGE_SIZE);
    let size = match pages.checked_mul(PAGE_SIZE) {
        Some(s) => s,
        None => return EINVAL,
    };
    let layout = match Layout::from_size_align(size, PAGE_SIZE) {
        Ok(l) => l,
        Err(_) => return EINVAL,
    };
    allocator::dealloc(addr as *mut u8, layout);
    0
}

/// changes memory permissions, make a page read-only, executable, etc. Musl calls during thread stack setup
pub fn sys_mprotect(addr: usize, len: usize, prot: usize) -> isize {
    if addr == 0 || len == 0 {
        return EINVAL;
    }
    if !addr.is_multiple_of(PAGE_SIZE) {
        return EINVAL;
    }
    let allowed_prot = PROT_NONE | PROT_READ | PROT_WRITE;
    if (prot & !allowed_prot) != 0 {
        return EINVAL;
    }
    0
}
