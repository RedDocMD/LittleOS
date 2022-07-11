pub mod bitmap_alloc;
pub mod fixed_buffer_alloc;

pub use core::alloc::{AllocError, Allocator, Layout};

// This is a complete stub for the global allocator.
// Do NOT use the global allocator for anything, it returns
// null for alloc and panics in dealloc.
struct StubGlobalAllocator;

unsafe impl core::alloc::GlobalAlloc for StubGlobalAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unreachable!(
            "dealloc on StubGlobalAllocator should never be called because it allocates only null"
        );
    }
}

#[global_allocator]
static STUB_GLOBAL_ALLOCATOR: StubGlobalAllocator = StubGlobalAllocator {};

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!(
        "Failed to allocate {} bytes with alignment of {}",
        layout.size(),
        layout.align()
    );
}
