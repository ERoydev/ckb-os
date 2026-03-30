use core::sync::atomic::{AtomicUsize, Ordering};

const PAGE_SIZE: usize = 4096;

static HEAP_START: AtomicUsize = AtomicUsize::new(0);
static HEAP_END: AtomicUsize = AtomicUsize::new(0);
static HEAP_POS: AtomicUsize = AtomicUsize::new(0);

/// Store heap bounds. Called once from `__platform_bootstrap()`.
pub fn init_memory(heap_start: usize, heap_end: usize) {
    let aligned = (heap_start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    HEAP_START.store(aligned, Ordering::Release);
    HEAP_POS.store(aligned, Ordering::Release);
    HEAP_END.store(heap_end, Ordering::Release);
}

const ENOMEM: isize = -12;

pub fn sys_brk(_addr: usize) -> isize {
    // fail so musl falls back to mmap
    ENOMEM
}

/// mmap — bump allocate from the heap region.
/// Only supports MAP_PRIVATE | MAP_ANONYMOUS.
pub fn sys_mmap(
    addr: usize,
    len: usize,
    _prot: usize,
    flags: usize,
    fd: usize,
    _offset: usize,
) -> isize {
    const MAP_ANONYMOUS: usize = 0x20;
    const MAP_PRIVATE: usize = 0x02;

    // Only support MAP_PRIVATE | MAP_ANONYMOUS
    if flags & MAP_ANONYMOUS == 0 || flags & MAP_PRIVATE == 0 || fd != usize::MAX {
        return ENOMEM;
    }

    // Ignore addr hint for anonymous mappings (unless MAP_FIXED, which we don't support)
    let _ = addr;

    let size = (len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    if size == 0 {
        return ENOMEM;
    }

    let pos = HEAP_POS.load(Ordering::Acquire);
    let end = HEAP_END.load(Ordering::Acquire);

    let new_pos = match pos.checked_add(size) {
        Some(p) if p <= end => p,
        _ => return ENOMEM,
    };

    HEAP_POS.store(new_pos, Ordering::Release);

    // Zero-fill the allocated region
    unsafe {
        core::ptr::write_bytes(pos as *mut u8, 0, size);
    }

    pos as isize
}

/// munmap — no-op in a zkVM (no page table to free).
pub fn sys_munmap(_addr: usize, _len: usize) -> isize {
    0
}

/// mprotect — no-op (no MMU).
pub fn sys_mprotect(_addr: usize, _len: usize, _prot: usize) -> isize {
    0
}
