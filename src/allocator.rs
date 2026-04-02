use core::alloc::Layout;
use zeroos_allocator_buddy::BUDDY_ALLOCATOR_OPS;

// Interface to the buddy allocator

pub fn init(heap_start: usize, heap_size: usize) {
    (BUDDY_ALLOCATOR_OPS.init)(heap_start, heap_size)
}

pub fn alloc(layout: Layout) -> *mut u8 {
    (BUDDY_ALLOCATOR_OPS.alloc)(layout)
}

pub fn dealloc(ptr: *mut u8, layout: Layout) {
    (BUDDY_ALLOCATOR_OPS.dealloc)(ptr, layout)
}

pub fn realloc(ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
    (BUDDY_ALLOCATOR_OPS.realloc)(ptr, old_layout, new_size)
}
