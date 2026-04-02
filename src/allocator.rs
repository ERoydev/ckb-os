use buddy_system_allocator::LockedHeap;
use core::alloc::Layout;

static HEAP: LockedHeap<32> = LockedHeap::empty();

pub fn init(heap_start: usize, heap_size: usize) {
    unsafe {
        HEAP.lock().init(heap_start, heap_size);
    }
}

pub fn alloc(layout: Layout) -> *mut u8 {
    HEAP.lock()
        .alloc(layout)
        .map(|nn| nn.as_ptr())
        .unwrap_or(core::ptr::null_mut())
}

pub fn dealloc(ptr: *mut u8, layout: Layout) {
    unsafe {
        HEAP.lock().dealloc(core::ptr::NonNull::new_unchecked(ptr), layout);
    }
}
